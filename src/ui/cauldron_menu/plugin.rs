use bevy::prelude::*;

use crate::state::{InGameState, MultiplayerGameState};
use crate::ui::plugin::ButtonActionSet;
use crate::ui::systems::{consume_mouse_on_exit, escape_to_running};

use super::components::IngredientSelection;
use super::systems;

/// Plugin that handles the cauldron ingredient selection UI.
///
/// Active in both single-player (`InGameState::CauldronMenu`) and multiplayer
/// (`MultiplayerGameState::CauldronMenu`). The brew/buff systems registered
/// by `CauldronPlugin` already apply on the host's MP simulation via
/// `is_gameplay_running`.
pub struct CauldronMenuPlugin;

impl Plugin for CauldronMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IngredientSelection>()
            // SP spawn/despawn
            .add_systems(
                OnEnter(InGameState::CauldronMenu),
                systems::spawn_cauldron_menu_ui,
            )
            .add_systems(
                OnExit(InGameState::CauldronMenu),
                (systems::despawn_cauldron_menu_ui, consume_mouse_on_exit),
            )
            // MP spawn/despawn (same handlers — UI is state-agnostic)
            .add_systems(
                OnEnter(MultiplayerGameState::CauldronMenu),
                systems::spawn_cauldron_menu_ui,
            )
            .add_systems(
                OnExit(MultiplayerGameState::CauldronMenu),
                (systems::despawn_cauldron_menu_ui, consume_mouse_on_exit),
            )
            .add_systems(
                Update,
                (
                    systems::button_action.in_set(ButtonActionSet),
                    escape_to_running,
                    systems::rebuild_menu_on_brew_state_change,
                    systems::respawn_menu_on_toggle,
                    systems::update_detail_panel_on_selection_change
                        .run_if(resource_changed::<IngredientSelection>),
                )
                    .run_if(
                        in_state(InGameState::CauldronMenu)
                            .or(in_state(MultiplayerGameState::CauldronMenu)),
                    ),
            );
    }
}
