//! Lightning Rod spell constants.

use bevy::prelude::*;

use crate::game::units::wizard::components::{PrimedSpell, Spell};

/// Primed Lightning Rod spell configuration.
pub const PRIMED_LIGHTNING_ROD: PrimedSpell = PrimedSpell {
    spell: Spell::LightningRod,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

// ===== Casting & Mana =====

/// Cast time before the lightning rod spawns (seconds).
pub const CAST_TIME: f32 = 1.5;

/// Mana cost to place the lightning rod.
pub const MANA_COST: f32 = 35.0;

// ===== Tower =====

/// How long the lightning rod persists on the battlefield (seconds).
pub const TOWER_DURATION: f32 = 15.0;

/// Visual height of the tower cylinder mesh (units).
pub const TOWER_HEIGHT: f32 = 60.0;

/// Visual radius of the tower cylinder mesh (units).
pub const TOWER_RADIUS: f32 = 8.0;

// ===== Lightning Strike =====

/// Seconds between lightning strikes on the rod.
pub const STRIKE_INTERVAL: f32 = 2.0;

/// Y position where the lightning bolt spawns (above camera view).
pub const STRIKE_SPAWN_HEIGHT: f32 = 2000.0;

/// How fast the lightning bolt travels downward (units/second).
pub const STRIKE_SPEED: f32 = 4000.0;

/// Visual width of the descending lightning bolt.
pub const STRIKE_BOLT_WIDTH: f32 = 10.0;

// ===== Arc Damage =====

/// Electric damage dealt per arc hit.
pub const ARC_DAMAGE: f32 = 15.0;

/// Radius to search for arc targets around the rod (units).
pub const ARC_RADIUS: f32 = 150.0;

/// Maximum number of units hit by arcs per strike.
pub const ARC_MAX_TARGETS: usize = 6;

/// How long arc visuals persist (seconds).
pub const ARC_LIFETIME: f32 = 0.3;

/// Visual width of the lightning arcs.
pub const ARC_WIDTH: f32 = 6.0;

// ===== Circle Indicator =====

/// Y position for the circle indicator on the ground.
pub const CIRCLE_Y_POSITION: f32 = 0.1;

// ===== Colors =====

/// Color of the arcs that jump to targets (electric blue).
pub const ARC_COLOR: Color = Color::srgb(0.7, 0.85, 1.0);

// ===== Talent Constants =====

// -- Tier 1 --

/// T1-0 Taller Rod: duration multiplier.
pub(super) const TALLER_ROD_DURATION_MULT: f32 = 1.5;

/// T1-1 Rapid Strikes: strike interval multiplier (lower = faster).
pub(super) const RAPID_STRIKES_INTERVAL_MULT: f32 = 0.65;

/// T1-2 Wider Arc: arc radius multiplier.
pub(super) const WIDER_ARC_RADIUS_MULT: f32 = 1.5;

/// T1-2 Wider Arc: extra targets per strike.
pub(super) const WIDER_ARC_EXTRA_TARGETS: usize = 3;

// -- Tier 2 --

/// T2-0 Chain Reaction: extra chain targets per arc.
pub(super) const CHAIN_REACTION_EXTRA_TARGETS: usize = 1;

/// T2-0 Chain Reaction: chained arc damage multiplier.
pub(super) const CHAIN_REACTION_DAMAGE_MULT: f32 = 0.5;

/// T2-1 Magnetic Field: slow modifier (negative = slower).
pub(super) const MAGNETIC_FIELD_SLOW: f32 = -0.4;

/// T2-1 Magnetic Field: slow duration in seconds.
pub(super) const MAGNETIC_FIELD_SLOW_DURATION: f32 = 2.0;

/// T2-2 Overcharge: triggers every N strikes.
pub(super) const OVERCHARGE_EVERY_N: u32 = 3;

/// T2-2 Overcharge: damage multiplier on overcharged strikes.
pub(super) const OVERCHARGE_DAMAGE_MULT: f32 = 2.5;

// -- Tier 3 --

/// T3-0 Storm Spire: damage multiplier for each rod.
pub(super) const STORM_SPIRE_DAMAGE_MULT: f32 = 0.6;

/// T3-0 Storm Spire: duration multiplier for each rod.
pub(super) const STORM_SPIRE_DURATION_MULT: f32 = 0.7;

/// T3-0 Storm Spire: offset distance between the two rods.
pub(super) const STORM_SPIRE_OFFSET: f32 = 40.0;

/// T3-2 Lightning Nexus: damage multiplier for each successive bonus strike (compounds).
pub(super) const LIGHTNING_NEXUS_FALLOFF: f32 = 0.5;

/// T3-1 Tesla Coil: damage ramp per strike (additive, e.g. 0.15 = +15%).
pub(super) const TESLA_COIL_RAMP_PER_STRIKE: f32 = 0.15;
