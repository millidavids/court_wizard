use bevy::prelude::*;

use crate::game::constants::UNIT_SCALE;

// King stats
pub const KING_HEALTH: f32 = 200.0; // Double standard 100
pub const KING_DAMAGE_PERCENTAGE: f32 = 1.0; // 100% bonus = double damage
pub const KING_RADIUS: f32 = 14.0 * UNIT_SCALE; // Larger than UNIT_RADIUS (8.0)
pub const KING_HITBOX_HEIGHT: f32 = 35.0 * UNIT_SCALE; // Taller than DEFENDER_HITBOX_HEIGHT (25.0)
pub const KING_MOVEMENT_SPEED: f32 = 115.0;

// Spell shield (multiplayer only)
/// Fraction of non-King defenders that must be alive for the shield to remain active.
pub const SPELL_SHIELD_THRESHOLD: f32 = 0.10;
/// Multiplayer anti-stall: the King's spell shield is force-removed after this
/// many seconds of match time, regardless of the kill threshold, so a player
/// can't keep-away/maze indefinitely while the shield never falls.
pub const MP_SPELL_SHIELD_MAX_DURATION_SECS: f32 = 90.0;
// Note: the dedicated shield visual (radius + color) was retired — the king's
// aura sphere now serves as the constant visual on both peers. See
// `spawn_king_aura_visual`.

// King-specific cohesion force constants
pub const KING_COHESION_BASE: f32 = 0.0; // No cohesion when no enemies inside aura
pub const KING_COHESION_THREATENED: f32 = 1.2; // Cohesion strength when enemies are inside aura

// Re-export aura constants from commander module for convenience
// Sprite animation
pub const KING_SPRITE_WIDTH: f32 = 36.0 * UNIT_SCALE; // Larger than infantry, ~0.75 aspect ratio
pub const KING_SPRITE_HEIGHT: f32 = 48.0 * UNIT_SCALE;
pub const KING_SPRITE_TINT: Color = Color::srgb(1.4, 1.15, 0.4); // Golden

pub use crate::game::units::commander::constants::{
    KING_AURA_DAMAGE_PERCENTAGE, KING_AURA_RADIUS, KING_AURA_SPEED_PERCENTAGE,
};
