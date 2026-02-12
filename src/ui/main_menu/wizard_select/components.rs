//! Wizard select screen specific components.

use bevy::prelude::*;

use crate::config::WizardType;

/// Marker component for entities that belong to the wizard select screen.
#[derive(Component)]
pub(super) struct OnWizardSelectScreen;

/// Actions that can be triggered by wizard select buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WizardSelectButtonAction {
    /// Select a wizard type (loads existing or creates new).
    SelectWizard(WizardType),
    /// Return to the landing screen.
    Back,
}
