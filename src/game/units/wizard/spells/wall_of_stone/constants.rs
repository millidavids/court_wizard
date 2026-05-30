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

/// Duration of the wall rising animation (seconds).
pub const WALL_RISE_DURATION: f32 = 0.6;

/// Interval between dust puff spawns during rise/sink.
pub(super) const WALL_DUST_INTERVAL: f32 = 0.1;

/// Number of dust puffs per spawn point along the wall.
pub(super) const WALL_DUST_PUFFS_PER_POINT: usize = 4;

/// Color for the wall preview during drag.
pub const WALL_PREVIEW_COLOR: Color = Color::srgba(0.55, 0.35, 0.15, 0.4);

/// Base health for all walls (flat, regardless of size).
pub const WALL_HEALTH: f32 = 500.0;

/// Damage each unit deals to a wall per attack cycle hit.
pub const WALL_DAMAGE_PER_HIT: f32 = 25.0;

/// Attack range for units hitting walls (world units from wall surface).
pub const WALL_ATTACK_RANGE: f32 = 30.0;

/// Wall color when at 0% HP (damage tint).
pub const WALL_DAMAGED_COLOR: Color = Color::srgba(0.5, 0.25, 0.15, 1.0);

// --- Tier 1 talent constants ---

/// Quarry Master: mana cost multiplier.
pub(super) const QUARRY_MASTER_MANA_MULT: f32 = 0.7;
/// Quarry Master: max wall length multiplier.
pub(super) const QUARRY_MASTER_LENGTH_MULT: f32 = 1.25;

/// Reinforced Stone: health multiplier.
pub(super) const REINFORCED_STONE_HEALTH_MULT: f32 = 2.0;
/// Reinforced Stone: width multiplier.
pub(super) const REINFORCED_STONE_WIDTH_MULT: f32 = 1.3;

/// Quick Foundations: mana cost multiplier per wall (2 walls placed).
pub(super) const QUICK_FOUNDATIONS_MANA_MULT: f32 = 0.6;

// --- Tier 2 talent constants ---

/// Jagged Stone: damage reflected back to attackers per hit.
pub(super) const JAGGED_STONE_REFLECT_DAMAGE: f32 = 5.0;

/// Permafrost Aura: radius around each wall that slows enemies.
pub(super) const PERMAFROST_AURA_RADIUS: f32 = 80.0;
/// Permafrost Aura: slow strength (negative = slower).
pub(super) const PERMAFROST_AURA_SLOW: f32 = -0.3;
/// Permafrost Aura: slow duration per tick (refreshed while in range).
pub(super) const PERMAFROST_AURA_SLOW_DURATION: f32 = 1.0;
/// Permafrost Aura: interval between slow application ticks.
pub(super) const PERMAFROST_AURA_TICK_INTERVAL: f32 = 0.5;

/// Living Stone: HP regeneration per second as fraction of max HP.
pub(super) const LIVING_STONE_REGEN_FRACTION: f32 = 0.05;
/// Living Stone: seconds without taking damage before regen starts.
pub(super) const LIVING_STONE_REGEN_DELAY: f32 = 3.0;

// --- Tier 3 talent constants ---

/// Collapsing Wall: explosion damage when wall is destroyed.
pub(super) const COLLAPSING_WALL_DAMAGE: f32 = 30.0;
/// Collapsing Wall: explosion radius.
pub(super) const COLLAPSING_WALL_RADIUS: f32 = 80.0;

/// Maze Architect: mana cost multiplier.
pub(super) const MAZE_ARCHITECT_MANA_MULT: f32 = 0.5;
/// Maze Architect: minimum wall count for bonus HP.
pub(super) const MAZE_ARCHITECT_WALL_THRESHOLD: usize = 3;
/// Maze Architect: bonus HP multiplier when threshold is met.
pub(super) const MAZE_ARCHITECT_HEALTH_MULT: f32 = 1.5;
