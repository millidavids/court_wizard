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
    /// Empowerment multiplier (for scaling unit-sized mesh).
    pub empowerment: f32,
}

impl ArcaneCrystalCircleIndicator {
    /// Creates a new circle indicator.
    pub fn new(position: Vec3, empowerment: f32) -> Self {
        Self {
            position,
            time_alive: 0.0,
            empowerment,
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

