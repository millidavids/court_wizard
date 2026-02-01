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

/// Actions that can be triggered by HUD buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HudButtonAction {
    OpenSpellBook,
}

/// Marker component for the level display text.
#[derive(Component)]
pub(super) struct LevelDisplay;

/// Marker component for the past victory display text.
#[derive(Component)]
pub(super) struct PastVictoryDisplay;
