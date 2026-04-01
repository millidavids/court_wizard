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

/// Color of the King's aura sphere (blue transparent)
pub const KING_AURA_COLOR: Color = Color::srgba(0.4, 0.4, 0.9, 0.08);
