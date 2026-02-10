//! Wizard select screen specific components.

use bevy::prelude::*;

use crate::config::WizardType;

/// Marker component for entities that belong to the wizard select screen.
#[derive(Component)]
pub(super) struct OnWizardSelectScreen;

/// Actions that can be triggered by wizard select buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WizardSelectButtonAction {
    /// Select a wizard type (moves to name input phase).
    SelectWizard(WizardType),
    /// Confirm the entered name and create the save.
    ConfirmName,
    /// Return to the landing screen (or back to wizard type selection).
    Back,
}

/// Resource tracking the wizard select flow state.
/// When a wizard type is selected, we move to the name input phase.
#[derive(Resource, Default)]
pub(super) struct WizardSelectState {
    /// The selected wizard type (None = still choosing type).
    pub selected_type: Option<WizardType>,
    /// The name being entered by the player.
    pub name_input: String,
    /// Error message to display (e.g., name taken, name empty).
    pub error_message: Option<String>,
}

/// Marker component for the text entity displaying the current name input.
#[derive(Component)]
pub(super) struct NameInputDisplay;

/// Marker component for the text entity displaying error messages.
#[derive(Component)]
pub(super) struct ErrorDisplay;
