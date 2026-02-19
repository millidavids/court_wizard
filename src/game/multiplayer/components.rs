//! Multiplayer-specific components.

use bevy::prelude::*;

/// Marker component for entities that belong to the multiplayer game screen.
///
/// Used for bulk cleanup when exiting the multiplayer game state.
#[derive(Component)]
pub struct OnMultiplayerGameScreen;
