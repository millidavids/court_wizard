//! Main menu plugin.
//!
//! Aggregates the Landing, Settings, Changelog, and Instructions plugins for the main menu flow.

use bevy::prelude::*;

use super::changelog::ChangelogPlugin;
use super::credits::CreditsPlugin;
use super::landing::plugin::LandingPlugin;
use super::multiplayer::plugin::MultiplayerPlugin;
use super::settings::plugin::SettingsPlugin;
use super::wizard_select::plugin::WizardSelectPlugin;
use crate::ui::compendium::MainMenuCompendiumPlugin;
use crate::ui::instructions::MainMenuInstructionsPlugin;

/// Main menu plugin that aggregates all main menu sub-screens.
///
/// This plugin contains:
/// - LandingPlugin (MenuState::Landing) - Start Game, Settings, Changelog, and Instructions buttons
/// - SettingsPlugin (MenuState::Settings) - Settings screen
/// - ChangelogPlugin (MenuState::Changelog) - Changelog screen
/// - MainMenuInstructionsPlugin (MenuState::Instructions) - Instructions screen
#[derive(Default)]
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            LandingPlugin,
            SettingsPlugin,
            ChangelogPlugin,
            CreditsPlugin,
            MainMenuInstructionsPlugin,
            MainMenuCompendiumPlugin,
            WizardSelectPlugin,
            MultiplayerPlugin,
        ));
    }
}
