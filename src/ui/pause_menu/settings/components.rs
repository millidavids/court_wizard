//! Pause menu settings screen specific components.

use bevy::prelude::*;

/// Marker component for entities that belong to the pause menu settings screen.
///
/// Used for cleanup when exiting the pause menu settings state.
#[derive(Component)]
pub struct OnPauseSettingsScreen;
