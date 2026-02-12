//! Pause menu plugin.

use bevy::prelude::*;

use super::main::plugin::PauseMainPlugin;
use super::settings::plugin::PauseSettingsPlugin;
use crate::ui::instructions::PauseMenuInstructionsPlugin;
use crate::ui::progress::PauseMenuProgressPlugin;

/// Plugin that manages all pause menu UI.
///
/// This plugin coordinates the pause menu main screen, settings screen, and instructions screen.
/// The settings screen reuses the main menu settings UI with pause-specific state transitions.
#[derive(Default)]
pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PauseMainPlugin,
            PauseSettingsPlugin,
            PauseMenuInstructionsPlugin,
            PauseMenuProgressPlugin,
        ));
    }
}
