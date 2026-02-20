//! Wizard select screen specific components.
//!
//! Shared components (DetailName, DetailDescription, DetailStatus, WizardCard,
//! SelectedWizardPreview) live in `wizard_select_shared`.

use bevy::prelude::*;

use crate::config::WizardType;

/// Marker component for entities that belong to the wizard select screen.
#[derive(Component)]
pub(super) struct OnWizardSelectScreen;

/// Actions that can be triggered by wizard select buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WizardSelectButtonAction {
    /// Preview a wizard type (show in detail panel).
    PreviewWizard(WizardType),
    /// Confirm selection and start the game with the previewed wizard.
    Play,
    /// Return to the landing screen.
    Back,
}
