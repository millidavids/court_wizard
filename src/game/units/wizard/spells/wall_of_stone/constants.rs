use bevy::prelude::*;

use crate::game::units::wizard::components::{PrimedSpell, Spell};

/// Primed Wall of Stone spell configuration.
pub const PRIMED_WALL_OF_STONE: PrimedSpell = PrimedSpell {
    spell: Spell::WallOfStone,
    cast_time: 0.0, // Instant start, wall placed on release
};

/// Mana cost for placing a wall.
pub const MANA_COST: f32 = 40.0;

/// Fixed width of the wall (perpendicular to drag direction).
pub const WALL_WIDTH: f32 = 40.0;

/// Height of the wall.
pub const WALL_HEIGHT: f32 = 80.0;

/// Total lifetime of the wall in seconds.
pub const WALL_DURATION: f32 = 20.0;

/// Duration of the sinking animation at end of life.
pub const WALL_SINK_DURATION: f32 = 2.0;

/// Minimum drag distance required to place a wall.
pub const MIN_WALL_LENGTH: f32 = 20.0;

/// Maximum wall length.
pub const MAX_WALL_LENGTH: f32 = 400.0;

/// Color for the placed wall.
pub const WALL_COLOR: Color = Color::srgba(0.75, 0.6, 0.45, 1.0);

/// Color for the wall preview during drag.
pub const WALL_PREVIEW_COLOR: Color = Color::srgba(0.55, 0.35, 0.15, 0.4);
