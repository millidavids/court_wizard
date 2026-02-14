//! Progress screen components.

use bevy::prelude::*;

/// Marker component for entities that belong to the progress screen.
#[derive(Component)]
pub(super) struct OnProgressScreen;

/// Marker component for the back button.
#[derive(Component)]
pub(super) struct BackButton;

/// Marker component for the clear progress button.
#[derive(Component)]
pub(super) struct ClearProgressButton;

/// Marker component for the scrollable progress container.
#[derive(Component)]
pub(super) struct ScrollableProgressContainer;

/// Marker for the confirmation popup.
#[derive(Component)]
pub(super) struct ConfirmationPopup;

/// Marker for confirm button in the popup.
#[derive(Component)]
pub(super) struct ConfirmClearButton;

/// Marker for cancel button in the popup.
#[derive(Component)]
pub(super) struct CancelClearButton;
