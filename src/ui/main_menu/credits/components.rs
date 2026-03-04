//! Components for credits screen.

use bevy::prelude::*;

/// Marker component for entities that should be despawned when leaving credits screen.
#[derive(Component)]
pub(super) struct OnCreditsScreen;

/// Marker component for the scrollable credits container.
#[derive(Component)]
pub(super) struct ScrollableCreditsContainer;

/// Marker component for the sprite credits button that opens CSV in browser.
#[derive(Component)]
pub(super) struct SpriteCreditsButton;
