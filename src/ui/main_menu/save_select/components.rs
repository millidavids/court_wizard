//! Save select screen specific components.

use bevy::prelude::*;

/// Marker component for entities that belong to the save select screen.
#[derive(Component)]
pub(super) struct OnSaveSelectScreen;

/// Actions that can be triggered by save select buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SaveSelectButtonAction {
    /// Load an existing save from the given slot.
    LoadSave(usize),
    /// Delete a save in the given slot.
    DeleteSave(usize),
    /// Return to the landing screen.
    Back,
}
