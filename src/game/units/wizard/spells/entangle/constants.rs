use crate::game::units::wizard::components::{PrimedSpell, Spell};

pub const PRIMED_ENTANGLE: PrimedSpell = PrimedSpell {
    spell: Spell::Entangle,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 2.0;
pub const MANA_COST: f32 = 25.0;
pub const CIRCLE_RADIUS: f32 = 120.0;
pub const ROOT_DURATION: f32 = 15.0;
pub const CIRCLE_Y_POSITION: f32 = 1.0;

// --- Tier 1 talent constants ---

/// Deep Roots: root duration multiplier.
pub(super) const DEEP_ROOTS_DURATION_MULT: f32 = 1.5;

/// Sprawling Thicket: radius multiplier.
pub(super) const SPRAWLING_THICKET_RADIUS_MULT: f32 = 1.4;
/// Sprawling Thicket: mana cost multiplier.
pub(super) const SPRAWLING_THICKET_MANA_MULT: f32 = 1.25;

/// Efficient Growth: mana cost multiplier.
pub(super) const EFFICIENT_GROWTH_MANA_MULT: f32 = 0.6;
/// Efficient Growth: cast time multiplier.
pub(super) const EFFICIENT_GROWTH_CAST_TIME_MULT: f32 = 0.7;

// --- Tier 2 talent constants ---

/// Thorny Vines: damage per second to rooted enemies.
pub(super) const THORNY_VINES_DPS: f32 = 3.0;
/// Thorny Vines: tick interval in seconds.
pub(super) const THORNY_VINES_TICK_INTERVAL: f32 = 0.5;

/// Clinging Roots: slow strength after root expires (negative = slower).
pub(super) const CLINGING_ROOTS_SLOW: f32 = -0.4;
/// Clinging Roots: slow duration after root expires.
pub(super) const CLINGING_ROOTS_SLOW_DURATION: f32 = 3.0;

/// Nourishing Roots: mana regenerated per second per rooted enemy.
pub(super) const NOURISHING_ROOTS_MANA_PER_SEC: f32 = 3.0;

// --- Tier 3 talent constants ---

/// Overgrowth: how much larger the zone grows (fraction of initial radius).
pub(super) const OVERGROWTH_GROWTH_FRACTION: f32 = 0.5;
/// Overgrowth: interval between checking for new units in expanding zone.
pub(super) const OVERGROWTH_CHECK_INTERVAL: f32 = 0.5;

/// Nature's Sanctuary: temporary HP granted to defenders in the area.
pub(super) const SANCTUARY_TEMP_HP: f32 = 15.0;

/// Stranglehold: minimum root duration before burst triggers.
pub(super) const STRANGLEHOLD_THRESHOLD: f32 = 3.0;
/// Stranglehold: burst damage when root expires on enemies.
pub(super) const STRANGLEHOLD_DAMAGE: f32 = 25.0;

// --- Vine VFX constants ---

/// Number of vine rings to spawn per entangle cast.
pub(super) const VINE_COUNT: usize = 50;
/// Minimum vine ring scale (radius in world units).
pub(super) const VINE_MIN_SCALE: f32 = 12.0;
/// Maximum vine ring scale (radius in world units).
pub(super) const VINE_MAX_SCALE: f32 = 32.0;
/// Maximum height the top of a vine ring can reach above ground.
/// Vines are positioned so at most 75% of the ring is above ground.
pub(super) const VINE_MAX_ABOVE_GROUND: f32 = 8.0;
/// Duration of the vine rising animation (seconds).
pub(super) const VINE_RISE_DURATION: f32 = 0.5;
/// How far below ground vines start before rising.
pub(super) const VINE_START_OFFSET: f32 = -15.0;
