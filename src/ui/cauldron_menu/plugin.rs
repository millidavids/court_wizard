use bevy::prelude::*;

use crate::state::InGameState;
use crate::ui::plugin::ButtonActionSet;
use crate::ui::systems::{consume_mouse_on_exit, escape_to_running};

use super::components::IngredientSelection;
use super::systems;

/// Plugin that handles the cauldron ingredient selection UI.
pub struct CauldronMenuPlugin;

impl Plugin for CauldronMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IngredientSelection>()
            .add_systems(
                OnEnter(InGameState::CauldronMenu),
                systems::spawn_cauldron_menu_ui,
            )
            .add_systems(
                OnExit(InGameState::CauldronMenu),
                (systems::despawn_cauldron_menu_ui, consume_mouse_on_exit),
            )
            .add_systems(
                Update,
                (
                    systems::button_action.in_set(ButtonActionSet),
                    escape_to_running,
                    systems::rebuild_menu_on_brew_state_change,
                    systems::respawn_menu_on_toggle,
                )
                    .run_if(in_state(InGameState::CauldronMenu)),
            );
    }
}
