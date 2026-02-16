use bevy::prelude::*;

// ============================================================================
// King Commander Aura Constants
// ============================================================================

/// Radius of the King's aura effect
pub const KING_AURA_RADIUS: f32 = 200.0;

/// King's aura damage buff percentage (+50% damage to defenders)
pub const KING_AURA_DAMAGE_PERCENTAGE: f32 = 0.5;

/// King's aura speed buff percentage (+25% speed to defenders)
pub const KING_AURA_SPEED_PERCENTAGE: f32 = 0.25;

/// Color of the King's aura sphere (orange transparent)
pub const KING_AURA_COLOR: Color = Color::srgba(1.0, 0.6, 0.0, 0.05);

// ============================================================================
// Default Commander Constants (for future commander types)
// ============================================================================

/// Default aura radius for new commander types
#[allow(dead_code)]
pub const DEFAULT_COMMANDER_AURA_RADIUS: f32 = 200.0;

/// Default damage buff percentage for new commanders
#[allow(dead_code)]
pub const DEFAULT_AURA_DAMAGE_BUFF: f32 = 0.3;

/// Default speed buff percentage for new commanders
#[allow(dead_code)]
pub const DEFAULT_AURA_SPEED_BUFF: f32 = 0.2;
