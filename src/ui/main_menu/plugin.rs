//! Main menu plugin.
//!
//! Aggregates the Landing, Settings, and Changelog plugins for the main menu flow.

use bevy::prelude::*;

use super::changelog::ChangelogPlugin;
use super::landing::plugin::LandingPlugin;
use super::settings::plugin::SettingsPlugin;

/// Main menu plugin that aggregates all main menu sub-screens.
///
/// This plugin contains:
/// - LandingPlugin (MenuState::Landing) - Start Game, Settings, and Changelog buttons
/// - SettingsPlugin (MenuState::Settings) - Settings screen
/// - ChangelogPlugin (MenuState::Changelog) - Changelog screen
#[derive(Default)]
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((LandingPlugin, SettingsPlugin, ChangelogPlugin));
    }
}
