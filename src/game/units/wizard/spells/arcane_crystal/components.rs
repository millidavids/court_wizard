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
    /// Active persistent beam groups + target pairs (beam_entities, target_entity).
    /// Each group may contain multiple beams when forked talent is active.
    pub active_beams: Vec<(Vec<Entity>, Entity)>,
    /// Whether the crystal was hit by disintegrate last frame.
    pub hit_by_disintegrate: bool,
    /// The last spell type that hit the crystal (for auto-casting).
    pub remembered_spell: Option<RememberedSpell>,
    /// Timer for auto-casting the remembered spell.
    pub auto_cast_timer: f32,
    /// Active auto-cast disintegrate beam group + target pair (beam_entities, target_entity).
    /// Only used when remembered_spell is Disintegrate.
    pub auto_disintegrate_beam: Option<(Vec<Entity>, Entity)>,
    /// Set to `true` on the frame a spell is absorbed; cleared each frame.
    /// Used by Crystal Network to detect fresh absorptions without pulse_timer ambiguity.
    pub just_absorbed: bool,
    /// Talent: damage multiplier for sub-projectiles (Refined Facets).
    pub damage_mult: f32,
    /// Talent: count multiplier for sub-projectiles (Overcharged Matrix).
    pub count_mult: f32,
    /// Talent: whether Spell Echo is active (30% chance to duplicate absorptions).
    pub spell_echo: bool,
    /// Whether this crystal is permanent (Auto-Crystal talent turret).
    /// Permanent crystals ignore lifetime, don't absorb spells, just fire magic missiles.
    pub permanent: bool,
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
            just_absorbed: false,
            damage_mult: 1.0,
            count_mult: 1.0,
            spell_echo: false,
            permanent: false,
        }
    }

    /// Triggers pulse animation.
    pub fn trigger_pulse(&mut self) {
        self.pulse_timer = super::constants::PULSE_DURATION;
    }

    /// Marks a spell absorption: triggers pulse and sets the one-shot flag
    /// consumed by Crystal Network chaining.
    pub fn mark_absorption(&mut self) {
        self.trigger_pulse();
        self.just_absorbed = true;
    }
}

/// Marker for the range indicator circle entity linked to a crystal.
#[derive(Component)]
pub(super) struct CrystalRangeIndicator {
    /// The crystal entity this indicator belongs to.
    pub crystal_entity: Entity,
}

/// Talent parameters computed at cast time from active talent selections.
pub(crate) struct ArcaneCrystalTalentParams {
    // Tier 1
    pub damage_mult: f32,
    pub range_mult: f32,
    pub duration_mult: f32,
    // Tier 2
    pub count_mult: f32,
    pub resonance_cascade: bool,
    pub spell_echo: bool,
    // Tier 3
    pub crystal_network: bool,
    pub prismatic_explosion: bool,
    pub auto_crystal: bool,
}

impl Default for ArcaneCrystalTalentParams {
    fn default() -> Self {
        Self {
            damage_mult: 1.0,
            range_mult: 1.0,
            duration_mult: 1.0,
            count_mult: 1.0,
            resonance_cascade: false,
            spell_echo: false,
            crystal_network: false,
            prismatic_explosion: false,
            auto_crystal: false,
        }
    }
}

/// Resonance Cascade tracker — counts absorptions and triggers burst at threshold.
#[derive(Component)]
pub(crate) struct ResonanceCascade {
    /// Number of absorptions since last burst.
    pub absorptions: u32,
}

/// Prismatic Explosion marker — crystal detonates on expiry.
#[derive(Component)]
pub(crate) struct PrismaticExplosion;

/// Auto-Crystal timer — fires projectiles at nearby enemies periodically.
#[derive(Component)]
pub(crate) struct AutoCrystalTimer {
    /// Seconds since last auto-fire.
    pub timer: f32,
}

/// Crystal Network marker — allows multiple crystals and chaining.
#[derive(Component)]
pub(crate) struct CrystalNetwork;

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
    /// Optional lifetime in seconds. When set, the entity is despawned after this duration.
    /// Used for one-shot beams (e.g. FoD burst) that aren't tracked by crystal code.
    pub lifetime: Option<f32>,
}
