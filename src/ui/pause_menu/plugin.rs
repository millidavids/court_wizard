//! Pause menu plugin.

use bevy::prelude::*;

use super::main::plugin::PauseMainPlugin;
use super::mp_settings::MpPauseSettingsPlugin;
use super::settings::plugin::PauseSettingsPlugin;
use crate::ui::compendium::PauseMenuCompendiumPlugin;
use crate::ui::manual::PauseMenuManualPlugin;

/// Plugin that manages all pause menu UI.
///
/// This plugin coordinates the pause menu main screen, settings screens (SP
/// and MP), and manual screen. The settings screens reuse the main menu
/// settings UI with state-specific transitions.
#[derive(Default)]
pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PauseMainPlugin,
            PauseSettingsPlugin,
            MpPauseSettingsPlugin,
            PauseMenuManualPlugin,
            PauseMenuCompendiumPlugin,
        ));
    }
}
