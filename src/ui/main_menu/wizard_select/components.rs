//! Wizard select screen specific components.

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

/// Resource tracking which wizard type is currently previewed in the detail panel.
#[derive(Resource)]
pub(super) struct SelectedWizardPreview(pub WizardType);

/// Marker for the detail panel name text.
#[derive(Component)]
pub(super) struct DetailName;

/// Marker for the detail panel description text.
#[derive(Component)]
pub(super) struct DetailDescription;

/// Marker for the detail panel status text.
#[derive(Component)]
pub(super) struct DetailStatus;

/// Marker for the detail panel container (for border color updates).
#[derive(Component)]
pub(super) struct DetailPanel;

/// Marker for a grid card, storing which wizard type it represents.
#[derive(Component)]
pub(super) struct WizardCard(pub WizardType);
