use bevy::prelude::*;

use crate::game::constants::{DEFENDER_BASE, TINT_BLUE, tint};

// King visual style
pub const KING_COLOR: Color = tint(DEFENDER_BASE, TINT_BLUE, 0.3); // Blue tint (commander)

// King stats
pub const KING_HEALTH: f32 = 100.0; // Double standard 50
pub const KING_DAMAGE_PERCENTAGE: f32 = 1.0; // 100% bonus = double damage
pub const KING_RADIUS: f32 = 14.0; // Larger than UNIT_RADIUS (8.0)
pub const KING_HITBOX_HEIGHT: f32 = 35.0; // Taller than DEFENDER_HITBOX_HEIGHT (25.0)
pub const KING_MOVEMENT_SPEED: f32 = 100.0; // Same as standard infantry

// King-specific cohesion force constants
pub const KING_COHESION_BASE: f32 = 0.0; // No cohesion when no enemies inside aura
pub const KING_COHESION_THREATENED: f32 = 1.2; // Cohesion strength when enemies are inside aura

// Re-export aura constants from commander module for convenience
pub use crate::game::units::commander::constants::{
    KING_AURA_COLOR, KING_AURA_DAMAGE_PERCENTAGE, KING_AURA_RADIUS, KING_AURA_SPEED_PERCENTAGE,
};
