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

/// Color of the lightning rod tower (metallic grey/silver).
pub const TOWER_COLOR: Color = Color::srgb(0.6, 0.6, 0.65);

/// Color of the descending lightning bolt (bright electric blue/white).
pub const BOLT_COLOR: Color = Color::srgb(0.8, 0.9, 1.0);

/// Color of the arcs that jump to targets (electric blue).
pub const ARC_COLOR: Color = Color::srgb(0.7, 0.85, 1.0);

/// Color of the flash at impact point.
#[allow(dead_code)]
pub const STRIKE_FLASH_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);

/// Color of the circle indicator during casting.
pub const CIRCLE_COLOR: Color = Color::srgba(0.7, 0.85, 1.0, 0.4);
