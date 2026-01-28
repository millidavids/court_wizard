use bevy::prelude::*;

/// Marker component for the HUD root container.
#[derive(Component)]
pub struct HudRoot;

/// Marker component for the mana bar fill element.
#[derive(Component)]
pub struct ManaBarFill;

/// Marker component for the cast bar fill element.
#[derive(Component)]
pub struct CastBarFill;

/// Actions that can be triggered by HUD buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudButtonAction {
    OpenSpellBook,
}

/// Marker component for the level display text.
#[derive(Component)]
pub struct LevelDisplay;

/// Marker component for the past victory display text.
#[derive(Component)]
pub struct PastVictoryDisplay;
