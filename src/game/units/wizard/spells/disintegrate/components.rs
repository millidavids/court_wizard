use bevy::prelude::*;

use super::constants;
use crate::game::units::DamageType;

/// Marker for the outer glow cylinder that follows a beam.
#[derive(Component)]
pub struct BeamGlow {
    /// The beam entity this glow tracks.
    pub beam_entity: Entity,
}

/// Marker for the bright flare sphere at a beam's origin.
#[derive(Component)]
pub struct BeamOriginFlare {
    /// The beam entity this flare tracks.
    pub beam_entity: Entity,
}

/// Elliptical ground shadow at the beam's impact point.
#[derive(Component)]
pub struct BeamEclipse {
    /// The beam entity this eclipse tracks.
    pub beam_entity: Entity,
}

/// A small particle emitted from the beam's impact point.
#[derive(Component)]
pub struct DisintegrateParticle {
    /// World-space velocity of the particle.
    pub velocity: Vec3,
    /// Seconds since this particle was spawned.
    pub time_alive: f32,
}

/// A smoke wisp that drifts upward off the beam and self-dissipates.
/// These are independent entities that persist after the beam despawns.
#[derive(Component)]
pub struct BeamSmoke {
    /// World-space velocity (primarily upward with slight lateral spread).
    pub velocity: Vec3,
    /// Seconds since this wisp was spawned.
    pub time_alive: f32,
}

/// Component for disintegrate beam.
///
/// The beam is a continuous ray that deals damage to entities along its path.
#[derive(Component)]
pub struct DisintegrateBeam {
    /// Origin point of the beam in world space.
    pub origin: Vec3,
    /// Direction the beam is pointing (normalized).
    pub direction: Vec3,
    /// Length of the beam.
    pub length: f32,
    /// Type of damage dealt.
    #[allow(dead_code)]
    pub damage_type: DamageType,
    /// Time since last damage tick.
    pub time_since_damage: f32,
    /// Time since beam was spawned (used for growth animation).
    pub time_alive: f32,
    /// Whether this beam is empowered.
    pub empowerment: f32,
    /// Optional damage override (used by crystal beams with custom damage).
    pub damage_per_tick_override: Option<f32>,

    // ── Talent fields ────────────────────────────────────────────────
    /// Width multiplier from talents (Focused Lens, Unfocused Beam, Annihilation Beam).
    pub width_multiplier: f32,
    /// Damage multiplier from talents (Focused Lens, Unfocused Beam, Forked, Annihilation, Sweeping).
    pub damage_multiplier: f32,
    /// Fan offset angle for this beam (Forked: -0.13, 0, +0.13).
    pub fan_offset_angle: f32,
    /// Whether damage escalates over channel time (Escalating Intensity).
    pub escalating: bool,
    /// Whether beam auto-sweeps (Sweeping Destruction).
    pub sweeping: bool,
    /// Current sweep angle offset from center direction.
    pub sweep_angle: f32,
    /// Current sweep direction: 1.0 or -1.0.
    pub sweep_direction: f32,
    /// The center direction the sweep oscillates around (set from cursor).
    pub sweep_center_direction: Vec3,
    /// Whether beam detonates on channeling end (Searing Finale).
    pub searing_finale: bool,
    /// Whether resonance pulses emit from beam (Unstable Resonance).
    pub resonance: bool,
    /// Timer for resonance pulse spawning.
    pub resonance_timer: f32,
    /// Whether this is an Annihilation sky beam (position locked on cast).
    pub annihilation: bool,
    /// Ground-level XZ position where an annihilation beam was initially cast.
    /// Used by sweep system to oscillate origin around this point.
    pub annihilation_cast_pos: Vec3,
    /// Scale factor for resonance mini-spells (1.0 for wizard, smaller for crystal beams).
    pub mini_spell_scale: f32,
    /// When true, collision checks project to Y=0 (XZ-only). Used by crystal beams
    /// whose origin is elevated but targets are at ground level.
    pub ground_collision: bool,
}

impl DisintegrateBeam {
    /// Creates a new disintegrate beam.
    ///
    /// # Arguments
    ///
    /// * `origin` - Starting position of the beam
    /// * `direction` - Direction the beam points (will be normalized)
    /// * `length` - Length of the beam
    /// * `empowerment` - Empowerment multiplier
    pub fn new(origin: Vec3, direction: Vec3, length: f32, empowerment: f32) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            length,
            damage_type: constants::DAMAGE_TYPE,
            time_since_damage: 0.0,
            time_alive: 0.0,
            empowerment,
            damage_per_tick_override: None,
            width_multiplier: 1.0,
            damage_multiplier: 1.0,
            fan_offset_angle: 0.0,
            escalating: false,
            sweeping: false,
            sweep_angle: 0.0,
            sweep_direction: 1.0,
            sweep_center_direction: Vec3::ZERO,
            searing_finale: false,
            resonance: false,
            resonance_timer: 0.0,
            annihilation: false,
            annihilation_cast_pos: Vec3::ZERO,
            mini_spell_scale: 1.0,
            ground_collision: false,
        }
    }

    /// Gets the damage per tick, using override if set, otherwise scaled by empowerment and talents.
    pub fn damage_per_tick(&self) -> f32 {
        let base = if let Some(override_damage) = self.damage_per_tick_override {
            override_damage * self.empowerment
        } else {
            constants::DAMAGE_PER_TICK * self.empowerment
        };
        let escalating_mult = if self.escalating {
            let t = (self.time_alive / constants::ESCALATING_RAMP_TIME).clamp(0.0, 1.0);
            constants::ESCALATING_START_MULT
                + t * (constants::ESCALATING_MAX_MULT - constants::ESCALATING_START_MULT)
        } else {
            1.0
        };
        base * self.damage_multiplier * escalating_mult
    }

    /// Gets the beam width, scaled by empowerment and talent multipliers.
    pub fn beam_width(&self) -> f32 {
        constants::BEAM_WIDTH * self.empowerment * self.width_multiplier
    }

    /// Checks if enough time has passed to deal damage again.
    pub fn should_damage(&self) -> bool {
        self.time_since_damage >= constants::DAMAGE_INTERVAL
    }

    /// Resets the damage timer.
    pub fn reset_damage_timer(&mut self) {
        self.time_since_damage = 0.0;
    }

    /// Updates the damage timer.
    pub fn update_damage_timer(&mut self, delta: f32) {
        self.time_since_damage += delta;
    }

    /// Updates the time alive counter.
    pub fn update_time_alive(&mut self, delta: f32) {
        self.time_alive += delta;
    }

    /// Gets the current animated length based on growth time.
    ///
    /// Beam grows from 0 to full length over BEAM_GROWTH_TIME seconds.
    pub fn current_length(&self) -> f32 {
        if self.time_alive >= constants::BEAM_GROWTH_TIME {
            self.length
        } else {
            let growth_factor = self.time_alive / constants::BEAM_GROWTH_TIME;
            self.length * growth_factor
        }
    }

    /// Checks if a point is within the beam.
    ///
    /// # Arguments
    ///
    /// * `point` - The point to check
    ///
    /// # Returns
    ///
    /// True if the point is within the beam's width and length.
    pub fn contains_point(&self, point: Vec3) -> bool {
        self.contains_point_with_radius(point, 0.0)
    }

    /// Checks if a unit with the given hitbox radius is hit by the beam.
    ///
    /// Full 3D point-to-line-segment distance check. The beam has uniform width
    /// along its entire length. When `ground_collision` is set, projects both beam
    /// and point to Y=0 for XZ-only checks (used by crystal beams).
    pub fn contains_point_with_radius(&self, point: Vec3, unit_radius: f32) -> bool {
        let current_len = self.current_length();
        if current_len < 0.001 {
            return false;
        }

        // For ground collision (crystal beams), flatten to XZ plane so the
        // elevated crystal origin doesn't prevent hitting ground-level units.
        let (origin, direction, check_point) = if self.ground_collision {
            let flat_origin = Vec3::new(self.origin.x, 0.0, self.origin.z);
            let flat_dir = Vec3::new(self.direction.x, 0.0, self.direction.z);
            let flat_dir = flat_dir.normalize_or(self.direction);
            let flat_point = Vec3::new(point.x, 0.0, point.z);
            (flat_origin, flat_dir, flat_point)
        } else {
            (self.origin, self.direction, point)
        };

        let to_point = check_point - origin;
        let proj = to_point.dot(direction);

        if proj < -unit_radius || proj > current_len + unit_radius {
            return false;
        }

        let clamped_proj = proj.clamp(0.0, current_len);
        let closest = origin + direction * clamped_proj;
        let distance = check_point.distance(closest);

        // Cone width at this point (0 at origin, full at tip)
        let cone_t = if current_len > 0.001 {
            clamped_proj / current_len
        } else {
            0.0
        };
        distance <= self.beam_width() * cone_t + unit_radius
    }

    /// Checks if the beam cylinder intersects a vertical hitbox cylinder.
    ///
    /// Uses separate horizontal (XZ) and vertical (Y) overlap checks:
    /// - Horizontally: XZ distance between closest approach points <= beam_width + hitbox_radius
    /// - Vertically: beam's Y range at closest approach overlaps the hitbox [0, hitbox_height]
    pub fn intersects_hitbox_cylinder(
        &self,
        unit_pos: Vec3,
        hitbox_radius: f32,
        _hitbox_height: f32,
    ) -> bool {
        let current_len = self.current_length();
        if current_len < 0.001 {
            return false;
        }

        // Full 3D cone-cylinder intersection.
        // Project unit center onto the beam axis and check perpendicular distance
        // against the cone radius at that depth.
        let beam_dir = self.direction;
        let to_unit = unit_pos - self.origin;
        let forward_dist = to_unit.dot(beam_dir);

        if forward_dist < -hitbox_radius || forward_dist > current_len + hitbox_radius {
            return false;
        }

        let clamped = forward_dist.clamp(0.0, current_len);
        let closest_on_axis = self.origin + beam_dir * clamped;
        let perp_dist = (unit_pos - closest_on_axis).length();

        // Cone radius widens linearly from 0 at origin to beam_width at tip
        let cone_t = clamped / current_len;
        let cone_radius = self.beam_width() * cone_t;

        perp_dist <= cone_radius + hitbox_radius
    }

    /// Computes the eclipse ellipse radii at the beam tip, clipped by spell range.
    ///
    /// Returns `None` if the eclipse is entirely outside range, or
    /// `(eclipse_center_xz, major_axis, minor_axis, clipped_major_radius, minor_radius)`.
    pub fn eclipse_geometry(&self, spell_range: f32) -> Option<(Vec3, Vec3, Vec3, f32, f32)> {
        let current_len = self.current_length();
        if current_len < 1.0 {
            return None;
        }

        let tip = self.origin + self.direction * current_len;
        let sphere_radius = self.beam_width();

        // Compute ellipse radii
        let elevation_sin = self.direction.y.abs();
        let major_radius = if elevation_sin > 0.05 {
            sphere_radius / elevation_sin
        } else {
            sphere_radius * 20.0
        };
        let minor_radius = sphere_radius;

        // Ground projection direction
        let ground_dir = Vec3::new(self.direction.x, 0.0, self.direction.z);
        let (major_axis, minor_axis) = if ground_dir.length_squared() > 0.001 {
            let major = ground_dir.normalize();
            let minor = Vec3::new(-major.z, 0.0, major.x);
            (major, minor)
        } else {
            (Vec3::X, Vec3::Z)
        };

        let eclipse_center = Vec3::new(tip.x, 0.0, tip.z);

        // Clip major radius by wizard's ground range.
        // Annihilation sky beams skip this — their cast position is already
        // range-clamped by the casting logic, and the sky-height origin would
        // make the formula produce zero.
        let clipped_major = if self.annihilation {
            major_radius
        } else {
            let origin_xz = Vec3::new(self.origin.x, 0.0, self.origin.z);
            let max_ground_range = if self.origin.y < spell_range {
                (spell_range * spell_range - self.origin.y * self.origin.y).sqrt()
            } else {
                0.0
            };
            let center_dist = eclipse_center.distance(origin_xz);
            let max_major = (max_ground_range - center_dist).max(0.0);
            major_radius.min(max_major)
        };

        if clipped_major < 0.01 {
            return None;
        }

        Some((
            eclipse_center,
            major_axis,
            minor_axis,
            clipped_major,
            minor_radius,
        ))
    }
}

/// Detonation line spawned by Searing Finale when channeling ends.
#[derive(Component)]
pub struct SearingFinaleDetonation {
    /// Origin of the detonation line.
    pub origin: Vec3,
    /// Direction of the detonation line.
    pub direction: Vec3,
    /// Length of the detonation line.
    pub length: f32,
    /// Half-width of the detonation hitbox.
    pub half_width: f32,
    /// Damage to deal.
    pub damage: f32,
    /// Time alive for visual animation.
    pub time_alive: f32,
    /// Whether damage has already been applied (one-shot).
    pub damage_applied: bool,
}
