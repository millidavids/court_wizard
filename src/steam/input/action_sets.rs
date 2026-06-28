//! Action-set activation based on game context.

use bevy::prelude::*;
use bevy_steamworks::Client;

use super::handles::SteamInputHandles;
use crate::state::{InGameState, MultiplayerGameState};
use crate::ui::focus::components::ModalOverlay;

/// Activates the right action set on every connected controller each frame:
/// `GameplayControls` while a match is actively running with no modal dialog up,
/// `MenuControls` everywhere else (menus, in-game overlays, modal dialogs, and the
/// wizard tower / study graph). `ActivateActionSet` is documented as cheap and
/// safe to call repeatedly.
pub(crate) fn activate_steam_action_sets(
    client: Res<Client>,
    handles: Res<SteamInputHandles>,
    sp_state: Option<Res<State<InGameState>>>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    modals: Query<(), With<ModalOverlay>>,
) {
    if !handles.resolved {
        return;
    }
    let sp_running = sp_state.is_some_and(|s| *s.get() == InGameState::Running);
    let mp_running = mp_state.is_some_and(|s| *s.get() == MultiplayerGameState::Running);
    // A modal dialog during gameplay must route South/East to UIConfirm/UIBack
    // (MenuControls), not Activate (GameplayControls), so it can be dismissed.
    let gameplay = (sp_running || mp_running) && modals.is_empty();

    let set = if gameplay {
        handles.gameplay_set
    } else {
        handles.menu_set
    };

    let input = client.input();
    for controller in input.get_connected_controllers() {
        input.activate_action_set_handle(controller, set);
    }
}
