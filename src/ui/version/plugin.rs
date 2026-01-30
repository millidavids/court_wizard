//! Plugin for version display.

use bevy::prelude::*;

use super::systems;
use crate::state::AppState;

/// Plugin that displays the version number in the bottom-left corner.
pub struct VersionPlugin;

impl Plugin for VersionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), systems::setup);
    }
}
