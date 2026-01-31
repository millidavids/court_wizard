//! Components for changelog screen.

use bevy::prelude::*;

/// Marker component for entities that should be despawned when leaving changelog screen.
#[derive(Component)]
pub struct OnChangelogScreen;

/// Marker component for the back button.
#[derive(Component)]
pub struct BackButton;

/// Marker component for the scrollable changelog container.
#[derive(Component)]
pub struct ScrollableChangelogContainer;
