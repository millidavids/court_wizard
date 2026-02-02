//! Instructions screen components.

use bevy::prelude::*;

/// Marker component for entities that belong to the instructions screen.
#[derive(Component)]
pub(super) struct OnInstructionsScreen;

/// Marker component for the back button.
#[derive(Component)]
pub(super) struct BackButton;

/// Marker component for the scrollable instructions container.
#[derive(Component)]
pub(super) struct ScrollableInstructionsContainer;
