//! Components for version display.

use bevy::prelude::*;

/// Marker component for the version text container.
#[derive(Component)]
pub(super) struct VersionText;

/// Marker component for the GitHub link button.
#[derive(Component)]
pub(super) struct GitHubButton;
