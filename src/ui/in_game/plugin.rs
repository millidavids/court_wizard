//! In-game UI plugin.

use bevy::prelude::*;

use crate::state::{AppState, InGameState};

use super::systems;

/// Plugin that manages in-game UI and input handling.
///
/// Registers systems for:
/// - HUD spawning and updates
/// - Keyboard input during active gameplay (e.g., pause on Escape)
#[derive(Default)]
pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), systems::spawn_hud)
            .add_systems(
                Update,
                (
                    systems::block_spell_input_on_button_interaction,
                    systems::keyboard_input,
                    systems::hud_button_action,
                    systems::update_mana_bar,
                    systems::update_cast_bar,
                )
                    .run_if(in_state(InGameState::Running)),
            );
    }
}
