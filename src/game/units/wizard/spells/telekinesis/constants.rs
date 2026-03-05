use bevy::prelude::*;

use crate::game::units::wizard::components::{PrimedSpell, Spell};

/// PrimedSpell constant for Telekinesis.
pub const PRIMED_TELEKINESIS: PrimedSpell = PrimedSpell {
    spell: Spell::Telekinesis,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

/// Cast time for Telekinesis in seconds (very short).
pub const CAST_TIME: f32 = 0.5;

/// Mana cost for casting Telekinesis.
pub const MANA_COST: f32 = 10.0;

/// Maximum distance from cursor to find a nearby drop (units).
pub const PICKUP_RADIUS: f32 = 100.0;

/// Y position of the indicator ring (slightly above ground).
pub const INDICATOR_Y: f32 = 2.0;

/// Radius of the indicator ring around the targeted drop.
pub const INDICATOR_RADIUS: f32 = 15.0;

// --- Talent constants ---

// T1: Quick Grab
/// Near-instant cast time when Quick Grab talent is active.
pub const QUICK_GRAB_CAST_TIME: f32 = 0.05;

// T1: Mana Efficiency
/// Mana cost multiplier when Mana Efficiency talent is active.
pub const MANA_EFFICIENCY_COST_MULT: f32 = 0.5;

// T2: Magnetic Pull
/// Speed at which ingredients drift toward the wizard (units/sec).
pub const MAGNETIC_PULL_SPEED: f32 = 30.0;
/// Maximum range from wizard for magnetic pull to affect drops.
pub const MAGNETIC_PULL_RADIUS: f32 = 300.0;

// T2: Harvest
/// Damage dealt to enemies near a pickup point.
pub const HARVEST_DAMAGE: f32 = 40.0;
/// Radius around pickup point for Harvest damage.
pub const HARVEST_RADIUS: f32 = 40.0;
/// Duration of the light blue flash on enemies hit by Harvest.
pub const HARVEST_FLASH_DURATION: f32 = 0.2;
/// Color for the harvest flash overlay (light blue).
pub const HARVEST_FLASH_COLOR: Color = Color::srgba(0.4, 0.75, 1.0, 0.7);

// T2: Keen Senses
/// Drop chance multiplier when Keen Senses talent is active.
pub const KEEN_SENSES_DROP_CHANCE_MULT: f64 = 1.5;

// T3: Telekinetic Storm
/// Mana cost multiplier per ingredient for Telekinetic Storm.
pub const STORM_MANA_COST_MULT: f32 = 3.0;

// T3: Transmutation
/// Brew potency increase per ingredient collected.
pub const TRANSMUTATION_POTENCY_PER_STACK: f32 = 0.1;

// T3: Psychic Shockwave
/// Knockback speed for Psychic Shockwave.
pub const SHOCKWAVE_KNOCKBACK_SPEED: f32 = 200.0;
/// Duration of knockback from Psychic Shockwave.
pub const SHOCKWAVE_KNOCKBACK_DURATION: f32 = 0.3;
/// Duration for the shockwave torus to expand to max radius.
pub const SHOCKWAVE_EXPAND_DURATION: f32 = 0.5;
/// Maximum radius the shockwave torus expands to.
pub const SHOCKWAVE_MAX_RADIUS: f32 = 500.0;
/// Tube thickness ratio for the shockwave torus mesh.
pub const SHOCKWAVE_TORUS_MINOR: f32 = 0.15;
/// Color for the shockwave torus ring (light blue).
pub const SHOCKWAVE_COLOR: Color = Color::srgba(0.5, 0.8, 1.0, 0.6);
