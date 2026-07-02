//! Action-set activation based on game context.

use bevy::prelude::*;
use bevy_steamworks::Client;

use super::handles::{SteamInputHandles, connected_controllers};
use crate::state::{InGameState, MultiplayerGameState};
use crate::ui::focus::components::ModalOverlay;

/// Frames after handle resolution during which we force the active set to win
/// Steam's config-load race (see the toggle below). ~5s at 60fps — long enough
/// to outlast the config load, short enough to be invisible.
const CONFIG_LOAD_RACE_GRACE_FRAMES: u32 = 300;

/// Activates the right action set on every connected controller each frame:
/// `GameplayControls` while a match is actively running with no modal dialog up,
/// `MenuControls` everywhere else (menus, in-game overlays, modal dialogs, and the
/// wizard tower / study graph). `ActivateActionSet` is documented as cheap and
/// safe to call repeatedly, so we assert the set unconditionally every frame —
/// that also self-heals a controller connected mid-session onto the correct set.
pub(crate) fn activate_steam_action_sets(
    client: Res<Client>,
    handles: Res<SteamInputHandles>,
    sp_state: Option<Res<State<InGameState>>>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    modals: Query<(), With<ModalOverlay>>,
    mut frames: Local<u32>,
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
    let controllers = connected_controllers(&input);

    // On config load Steam auto-activates the config's default set (the manifest's
    // first action set) and DEDUPS repeated same-set calls — so our first
    // ActivateActionSet can lose that race and never re-apply, stranding the
    // landing menu on the wrong set. For a grace window after resolution, force our
    // set to win by toggling (activate the other set, then ours); the poll runs
    // after us in the same frame, so it never observes the intermediate set.
    let force = *frames < CONFIG_LOAD_RACE_GRACE_FRAMES;
    let other = if set == handles.gameplay_set {
        handles.menu_set
    } else {
        handles.gameplay_set
    };
    for &c in &controllers {
        if force {
            input.activate_action_set_handle(c, other);
        }
        input.activate_action_set_handle(c, set);
    }
    *frames = frames.saturating_add(1);
}
