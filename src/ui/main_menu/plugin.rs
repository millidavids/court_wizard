//! Main menu plugin.
//!
//! Aggregates the Landing and Settings plugins for the main menu flow.

use bevy::prelude::*;

use super::landing::plugin::LandingPlugin;
use super::settings::plugin::SettingsPlugin;

/// Main menu plugin that aggregates all main menu sub-screens.
///
/// This plugin contains:
/// - LandingPlugin (MenuState::Landing) - Start Game and Settings buttons
/// - SettingsPlugin (MenuState::Settings) - Settings screen
#[derive(Default)]
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((LandingPlugin, SettingsPlugin));
    }
}
