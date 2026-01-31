use bevy::prelude::*;

// King visual style
pub const KING_COLOR: Color = Color::srgb(1.0, 0.6, 0.0); // Orange

// King stats
pub const KING_HEALTH: f32 = 100.0; // Double standard 50
pub const KING_DAMAGE_PERCENTAGE: f32 = 1.0; // 100% bonus = double damage
pub const KING_RADIUS: f32 = 14.0; // Larger than UNIT_RADIUS (8.0)
pub const KING_HITBOX_HEIGHT: f32 = 35.0; // Taller than DEFENDER_HITBOX_HEIGHT (25.0)
pub const KING_MOVEMENT_SPEED: f32 = 100.0; // Same as standard infantry

// Cohesion aura constants
pub const KING_AURA_RADIUS: f32 = 200.0; // Range within which defenders feel pull, receive buffs, and enemies are detected
pub const KING_COHESION_BASE: f32 = 0.0; // No cohesion when no enemies inside aura
pub const KING_COHESION_THREATENED: f32 = 1.2; // Cohesion strength when enemies are inside aura
pub const KING_AURA_DAMAGE_PERCENTAGE: f32 = 0.5; // 50% damage bonus for units in King's aura
pub const KING_AURA_SPEED_PERCENTAGE: f32 = 0.25; // 25% speed bonus for all units in King's aura (including King himself)
