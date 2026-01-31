//! Plugin for version display.

use bevy::prelude::*;

use super::systems;
use crate::state::{AppState, InGameState};

/// Plugin that displays the version number and GitHub link in the bottom-left corner.
pub struct VersionPlugin;

impl Plugin for VersionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), systems::setup)
            .add_systems(OnEnter(InGameState::Paused), systems::setup)
            .add_systems(OnExit(AppState::MainMenu), systems::cleanup)
            .add_systems(OnExit(InGameState::Paused), systems::cleanup)
            .add_systems(
                Update,
                (
                    systems::handle_github_button,
                    systems::update_github_button_style,
                ),
            );
    }
}
