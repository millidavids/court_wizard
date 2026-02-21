//! Arcane Crystal spell components.

use bevy::prelude::*;

/// The type of spell the crystal has most recently absorbed.
/// Used for auto-casting on a timer.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum RememberedSpell {
    Fireball,
    Disintegrate,
    FingerOfDeath,
    Meteor,
    MagicMissile,
    ChainLightning,
}

impl RememberedSpell {
    /// Returns the representative damage value for interval calculation.
    fn damage_value(self) -> f32 {
        match self {
            Self::MagicMissile => 5.0,
            Self::ChainLightning => 20.0,
            Self::Meteor => 25.0,
            Self::Fireball => 50.0,
            Self::FingerOfDeath => 1000.0,
            Self::Disintegrate => 0.0, // special case — constant beam
        }
    }

    /// Returns the auto-cast interval for this spell type.
    /// Disintegrate returns 0.0 (special case: constant beam, no interval).
    pub fn auto_cast_interval(self) -> f32 {
        if self == Self::Disintegrate {
            return 0.0;
        }
        let raw = super::constants::AUTO_CAST_BASE_INTERVAL
            * (self.damage_value() / super::constants::AUTO_CAST_REFERENCE_DAMAGE);
        raw.clamp(
            super::constants::AUTO_CAST_MIN_INTERVAL,
            super::constants::AUTO_CAST_MAX_INTERVAL,
        )
    }
}

/// Main crystal entity placed on the battlefield.
///
/// Absorbs incoming spell projectiles/beams and re-emits smaller versions
/// at random enemies within range.
#[derive(Component)]
pub(crate) struct ArcaneCrystal {
    /// World position of the crystal.
    pub position: Vec3,
    /// Range within which the crystal targets enemies and limits projectiles.
    pub range: f32,
    /// Total lifetime (seconds).
    pub duration: f32,
    /// Time since the crystal was placed.
    pub time_alive: f32,
    /// Collision radius for detecting incoming spell hits.
    pub collision_radius: f32,
    /// Empowerment multiplier.
    pub empowerment: f32,
    /// Current pulse animation timer (0.0 = no pulse).
    pub pulse_timer: f32,
    /// Set of FingerOfDeath beam entities already processed (prevents re-triggering).
    pub fod_beams_processed: Vec<Entity>,
    /// Set of FireballExplosion entities already processed (prevents re-triggering).
    pub explosions_processed: Vec<Entity>,
    /// Active persistent beam + target pairs (beam_entity, target_entity).
    pub active_beams: Vec<(Entity, Entity)>,
    /// Whether the crystal was hit by disintegrate last frame.
    pub hit_by_disintegrate: bool,
    /// The last spell type that hit the crystal (for auto-casting).
    pub remembered_spell: Option<RememberedSpell>,
    /// Timer for auto-casting the remembered spell.
    pub auto_cast_timer: f32,
    /// Active auto-cast disintegrate beam + target pair (beam_entity, target_entity).
    /// Only used when remembered_spell is Disintegrate.
    pub auto_disintegrate_beam: Option<(Entity, Entity)>,
}

impl ArcaneCrystal {
    /// Creates a new Arcane Crystal.
    pub fn new(
        position: Vec3,
        range: f32,
        duration: f32,
        collision_radius: f32,
        empowerment: f32,
    ) -> Self {
        Self {
            position,
            range,
            duration,
            time_alive: 0.0,
            collision_radius,
            empowerment,
            pulse_timer: 0.0,
            fod_beams_processed: Vec::new(),
            explosions_processed: Vec::new(),
            active_beams: Vec::new(),
            hit_by_disintegrate: false,
            remembered_spell: None,
            auto_cast_timer: 0.0,
            auto_disintegrate_beam: None,
        }
    }

    /// Triggers pulse animation.
    pub fn trigger_pulse(&mut self) {
        self.pulse_timer = super::constants::PULSE_DURATION;
    }
}

/// Marker for the range indicator circle entity linked to a crystal.
#[derive(Component)]
pub(super) struct CrystalRangeIndicator {
    /// The crystal entity this indicator belongs to.
    pub crystal_entity: Entity,
}

/// Circle indicator shown during casting.
#[derive(Component)]
pub(super) struct ArcaneCrystalCircleIndicator {
    /// Position of the circle center.
    pub position: Vec3,
    /// Time this indicator has been active (for animations).
    pub time_alive: f32,
}

impl ArcaneCrystalCircleIndicator {
    /// Creates a new circle indicator.
    pub const fn new(position: Vec3) -> Self {
        Self {
            position,
            time_alive: 0.0,
        }
    }

    /// Returns the current scale factor for pulse animation.
    pub fn pulse_scale(&self) -> f32 {
        let pulse_freq = 2.0;
        let pulse_amplitude = 0.05;
        1.0 + (self.time_alive * pulse_freq * std::f32::consts::TAU).sin() * pulse_amplitude
    }
}

/// Marker component added to spell entities emitted by a crystal.
///
/// Carries the crystal's origin position and range for range-limiting.
/// Existing spell movement/collision systems handle the entity normally;
/// a separate system despawns entities that exceed the crystal's range.
#[derive(Component)]
pub(crate) struct CrystalSpawn {
    /// Origin position (crystal location) for range calculations.
    pub origin: Vec3,
    /// Maximum distance from origin before despawning.
    pub max_range: f32,
}

/// Brief beam burst emitted by the crystal toward a target.
///
/// Unlike DisintegrateBeam (which is channeled), this is a one-shot beam
/// that persists for a short duration then despawns.
#[derive(Component)]
pub(crate) struct CrystalBeam {
    /// Origin point of the beam.
    pub origin: Vec3,
    /// Normalized direction vector.
    pub direction: Vec3,
    /// Full beam length (clamped to crystal range).
    pub length: f32,
    /// Damage per tick.
    pub damage_per_tick: f32,
    /// Time between damage ticks.
    pub damage_interval: f32,
    /// Time since last damage tick.
    pub time_since_damage: f32,
    /// Time since beam was spawned (for growth animation).
    pub time_alive: f32,
    /// Total beam duration before despawn.
    pub beam_duration: f32,
    /// Beam width for collision detection.
    pub beam_width: f32,
    /// Empowerment multiplier.
    pub empowerment: f32,
    /// If true, beam never expires on its own (managed externally).
    pub persistent: bool,
}

impl CrystalBeam {
    /// Gets the current animated length (grows from 0 to full length quickly).
    pub fn current_length(&self) -> f32 {
        let growth_time = super::constants::BEAM_DURATION * 0.2; // 20% of duration for growth
        if self.time_alive >= growth_time {
            self.length
        } else {
            self.length * (self.time_alive / growth_time)
        }
    }

    /// Retargets the beam toward a new position.
    pub fn retarget(&mut self, origin: Vec3, target: Vec3, max_range: f32) {
        let direction = (target - origin).normalize();
        let distance = origin.distance(target);
        self.origin = origin;
        self.direction = direction;
        self.length = distance.min(max_range);
    }

    /// Checks if a point is within the beam.
    pub fn contains_point(&self, point: Vec3) -> bool {
        let to_point = point - self.origin;
        let projection_length = to_point.dot(self.direction);

        let current_len = self.current_length();
        if projection_length < 0.0 || projection_length > current_len {
            return false;
        }

        let closest_point_on_beam = self.origin + self.direction * projection_length;
        let distance_from_beam = point.distance(closest_point_on_beam);

        distance_from_beam <= self.beam_width
    }
}

/// Lightning arc visual emitted by the crystal.
#[derive(Component)]
pub(crate) struct CrystalLightningArc {
    /// Time remaining before arc despawns.
    pub lifetime: f32,
    /// Time since arc was created (for animation).
    pub time_alive: f32,
}
