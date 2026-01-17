//! Pause menu plugin.

use bevy::prelude::*;

use super::main::plugin::PauseMainPlugin;
use super::settings::plugin::PauseSettingsPlugin;

/// Plugin that manages all pause menu UI.
///
/// This plugin coordinates the pause menu main screen and settings screen.
#[derive(Default)]
pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PauseMainPlugin, PauseSettingsPlugin));
    }
}
