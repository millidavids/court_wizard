use bevy::prelude::*;

/// Marker component for the HUD root container.
#[derive(Component)]
pub(super) struct HudRoot;

/// Marker component for the mana bar fill element.
#[derive(Component)]
pub(crate) struct ManaBarFill;

/// Marker component for the cast bar fill element.
#[derive(Component)]
pub(super) struct CastBarFill;

/// Marker component for the brewing/reload overlay container on the cast bar.
#[derive(Component)]
pub(super) struct BrewingOverlay;

/// Marker component for the text inside the brewing/reload overlay.
#[derive(Component)]
pub(super) struct BrewingOverlayText;

/// Actions that can be triggered by HUD buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HudButtonAction {
    OpenSpellBook,
    OpenCauldronMenu,
}

/// Marker component for the level display text.
#[derive(Component)]
pub(super) struct LevelDisplay;

/// Marker component for the past victory display text.
#[derive(Component)]
pub(super) struct PastVictoryDisplay;

/// Marker component for the level clock display text.
#[derive(Component)]
pub(super) struct LevelClockDisplay;

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
pub(crate) struct KingHealthBarFill;

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
pub(crate) struct WaveDisplay;

/// Marker component for the "Wave incoming!" flash notification.
#[derive(Component)]
pub(super) struct WaveIncomingFlash {
    /// Time remaining for the flash (seconds).
    pub timer: f32,
}

/// Marker component for the buff tracker container row.
#[derive(Component)]
pub(super) struct BuffTrackerContainer;

/// Marker component for a single buff box, indexed into `CauldronBuffs::active_buffs`.
#[derive(Component)]
pub(super) struct BuffTrackerBox(pub usize);

/// Marker component for a buff timer text, indexed into `CauldronBuffs::active_buffs`.
#[derive(Component)]
pub(super) struct BuffTimerText(pub usize);

/// Marker component for a buff tooltip popup.
#[derive(Component)]
pub(super) struct BuffTooltip;

/// Marker component for the ammo display container (replaces mana bar for gunslinger).
#[derive(Component)]
pub(super) struct AmmoDisplayContainer;

/// Marker component for an individual ammo piece in the ammo display.
#[derive(Component)]
pub(super) struct AmmoPiece {
    pub index: u32,
}

/// Marker component for the ammo counter text (e.g., "12 / 60").
#[derive(Component)]
pub(super) struct AmmoCounterText;
