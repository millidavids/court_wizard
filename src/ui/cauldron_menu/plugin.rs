use bevy::prelude::*;

use crate::state::InGameState;
use crate::ui::plugin::ButtonActionSet;

use super::systems;

/// Plugin that handles the cauldron brew selection UI.
pub struct CauldronMenuPlugin;

impl Plugin for CauldronMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InGameState::CauldronMenu),
            systems::spawn_cauldron_menu_ui,
        )
        .add_systems(
            OnExit(InGameState::CauldronMenu),
            (
                systems::despawn_cauldron_menu_ui,
                systems::consume_mouse_on_exit,
            ),
        )
        .add_systems(
            Update,
            (
                systems::button_action.in_set(ButtonActionSet),
                systems::keyboard_input,
            )
                .run_if(in_state(InGameState::CauldronMenu)),
        );
    }
}
