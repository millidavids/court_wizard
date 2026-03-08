use bevy::prelude::*;

use crate::game::units::wizard::components::{PrimedSpell, Spell};

/// Primed Wall of Stone spell configuration.
pub const PRIMED_WALL_OF_STONE: PrimedSpell = PrimedSpell {
    spell: Spell::WallOfStone,
    cast_time: 0.0, // Instant start, wall placed on release
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

/// Mana cost for placing a wall.
pub const MANA_COST: f32 = 40.0;

/// Fixed width of the wall (perpendicular to drag direction).
pub const WALL_WIDTH: f32 = 40.0;

/// Height of the wall.
pub const WALL_HEIGHT: f32 = 80.0;

/// Duration of the sinking animation at end of life (used for non-permanent multiplayer walls).
pub const WALL_SINK_DURATION: f32 = 2.0;

/// Minimum drag distance required to place a wall.
pub const MIN_WALL_LENGTH: f32 = 20.0;

/// Maximum wall length.
pub const MAX_WALL_LENGTH: f32 = 400.0;

/// Color for the wall preview during drag.
pub const WALL_PREVIEW_COLOR: Color = Color::srgba(0.55, 0.35, 0.15, 0.4);

/// Base health for all walls (flat, regardless of size).
pub const WALL_HEALTH: f32 = 500.0;

/// Damage each unit deals to a wall per attack cycle hit.
pub const WALL_DAMAGE_PER_HIT: f32 = 25.0;

/// Attack range for units hitting walls (world units from wall surface).
pub const WALL_ATTACK_RANGE: f32 = 30.0;

/// Base wall color (matches SpellVisualAssets wall_of_stone material).
pub const WALL_BASE_COLOR: Color = Color::srgba(0.75, 0.6, 0.45, 1.0);

/// Wall color when at 0% HP (damage tint).
pub const WALL_DAMAGED_COLOR: Color = Color::srgba(0.5, 0.25, 0.15, 1.0);
