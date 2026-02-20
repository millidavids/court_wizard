//! In-game UI plugin.

use bevy::prelude::*;

use crate::game::run_conditions::is_gameplay_running;
use crate::state::AppState;
use crate::ui::plugin::ButtonActionSet;

use super::systems;

/// Plugin that manages in-game UI and input handling.
///
/// Registers systems for:
/// - HUD spawning on entering InGame state
/// - Keyboard input during active gameplay (e.g., pause on Escape)
#[derive(Default)]
pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), systems::spawn_hud)
            .add_systems(
                Update,
                systems::hud_button_action
                    .in_set(ButtonActionSet)
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                Update,
                (
                    systems::block_spell_input_on_button_interaction,
                    systems::keyboard_input,
                    systems::update_mana_bar,
                    systems::update_cast_bar,
                    systems::update_level_display,
                    systems::update_past_victory_display,
                )
                    .run_if(is_gameplay_running),
            );
    }
}
