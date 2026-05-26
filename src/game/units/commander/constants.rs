// (No `bevy::prelude` import needed after `KING_AURA_COLOR` was retired.)

// ============================================================================
// King Commander Aura Constants
// ============================================================================

/// Radius of the King's aura effect
pub const KING_AURA_RADIUS: f32 = 200.0;

/// King's aura damage buff percentage (+50% damage to defenders)
pub const KING_AURA_DAMAGE_PERCENTAGE: f32 = 0.5;

/// King's aura speed buff percentage (+25% speed to defenders)
pub const KING_AURA_SPEED_PERCENTAGE: f32 = 0.25;

// (Old `KING_AURA_COLOR` removed — the aura now uses the shared
// `king_aura_sphere` AuraSphereMaterial, which is parameterised in
// `visual_assets.rs` and rendered with a custom shader.)
