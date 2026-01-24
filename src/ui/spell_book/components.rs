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
