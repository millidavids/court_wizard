//! Main menu plugin.
//!
//! Aggregates the Landing, Settings, Manual, and other plugins for the main menu flow.

use bevy::prelude::*;

use super::background::BackgroundPlugin;
use super::landing::plugin::LandingPlugin;
use super::settings::plugin::SettingsPlugin;
use crate::ui::compendium::MainMenuCompendiumPlugin;
use crate::ui::manual::MainMenuManualPlugin;

/// Main menu plugin that aggregates all main menu sub-screens.
#[derive(Default)]
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            BackgroundPlugin,
            LandingPlugin,
            SettingsPlugin,
            MainMenuManualPlugin,
            MainMenuCompendiumPlugin,
        ));
    }
}
