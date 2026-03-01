use bevy::prelude::*;

/// Marker component for the HUD root container.
#[derive(Component)]
pub(super) struct HudRoot;

/// Marker component for the mana bar fill element.
#[derive(Component)]
pub(super) struct ManaBarFill;

/// Marker component for the cast bar fill element.
#[derive(Component)]
pub(super) struct CastBarFill;

/// Marker component for the brewing overlay text on the cast bar.
#[derive(Component)]
pub(super) struct BrewingOverlay;

/// Actions that can be triggered by HUD buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HudButtonAction {
    OpenSpellBook,
    OpenCauldronMenu,
}

/// Marker component for the level display text.
#[derive(Component)]
pub(super) struct LevelDisplay;

/// Marker component for the past victory display text.
#[derive(Component)]
pub(super) struct PastVictoryDisplay;

/// Marker component for the boss health bar root container.
#[derive(Component)]
pub(super) struct BossHealthBarRoot;

/// Marker component for the boss health bar fill element.
#[derive(Component)]
pub(super) struct BossHealthBarFill;

/// Marker component for the boss health bar text (percentage).
#[derive(Component)]
pub(super) struct BossHealthBarText;

/// Marker component for the king health bar fill element.
#[derive(Component)]
pub(super) struct KingHealthBarFill;

/// Marker component for a hag health bar section fill.
#[derive(Component)]
pub(super) struct HagHealthBarFill {
    pub identity: crate::game::units::boss::hags::components::HagIdentity,
}

/// Marker component for a hag health bar section text.
#[derive(Component)]
pub(super) struct HagHealthBarText {
    pub identity: crate::game::units::boss::hags::components::HagIdentity,
}

/// Marker component for the wave counter display text.
#[derive(Component)]
pub(super) struct WaveDisplay;

/// Marker component for the "Wave incoming!" flash notification.
#[derive(Component)]
pub(super) struct WaveIncomingFlash {
    /// Time remaining for the flash (seconds).
    pub timer: f32,
}
