use bevy::prelude::*;

/// Actions that can be triggered by spell book buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellBookButtonAction {
    MagicMissile,
    Disintegrate,
    Close,
}

/// Marker component for entities that should be cleaned up when exiting spell book.
#[derive(Component)]
pub struct OnSpellBookScreen;

/// Component storing original button colors for hover/pressed effects.
#[derive(Component)]
pub struct ButtonColors {
    pub background: Color,
    pub border: Color,
}
