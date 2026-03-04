//! Components for changelog screen.

use bevy::prelude::*;

/// Marker component for entities that should be despawned when leaving changelog screen.
#[derive(Component)]
pub(super) struct OnChangelogScreen;

/// Marker component for the scrollable changelog container.
#[derive(Component)]
pub(super) struct ScrollableChangelogContainer;
