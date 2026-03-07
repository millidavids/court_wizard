//! Multiplayer screen plugin.

use bevy::prelude::*;

use crate::game::multiplayer::components::PendingRematch;
use crate::state::MenuState;
use crate::ui::plugin::ButtonActionSet;

use super::systems::{
    button_action, cleanup_multiplayer_resources, process_lobby_messages, setup, update_ui_state,
};

/// Plugin that manages the multiplayer lobby screen UI.
#[derive(Default)]
pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Multiplayer), setup)
            .add_systems(
                OnExit(MenuState::Multiplayer),
                (
                    crate::ui::systems::cleanup_screen::<super::components::OnMultiplayerScreen>,
                    cleanup_multiplayer_resources,
                ),
            )
            .add_systems(
                Update,
                (
                    button_action.in_set(ButtonActionSet),
                    process_lobby_messages,
                    update_ui_state,
                )
                    .chain()
                    .run_if(in_state(MenuState::Multiplayer)),
            )
            // Auto-redirect to multiplayer lobby on rematch
            .add_systems(
                OnEnter(MenuState::Landing),
                auto_enter_multiplayer_on_rematch,
            );
    }
}

/// When returning to the main menu with a `PendingRematch`, skip the landing page
/// and go straight to the multiplayer lobby.
fn auto_enter_multiplayer_on_rematch(
    mut next_menu: ResMut<NextState<MenuState>>,
    pending: Option<Res<PendingRematch>>,
) {
    if pending.is_some() {
        next_menu.set(MenuState::Multiplayer);
    }
}
