//! Steam Input action-set and action handles.
//!
//! Handles are resolved once from the loaded IGA manifest and cached here. Steam's
//! `InputActionSetHandle_t` / `InputDigitalActionHandle_t` / `InputAnalogActionHandle_t`
//! are transparent `u64` aliases, so we store plain `u64` (0 = unresolved) and pass
//! them straight back into the `Input` API.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_steamworks::Client;
use steamworks::Input;

use crate::game::input::action_state::{AnalogAction, GamepadAction};

/// The real (non-zero) connected controllers. `Input::get_connected_controllers`
/// pads its return to `STEAM_INPUT_MAX_COUNT` with zero handles (steamworks-rs
/// 0.12.2 uses `Vec::shrink_to`, which trims capacity not length), so any caller
/// that indexes `[0]`, checks `is_empty`, or iterates must drop the padding first.
pub(crate) fn connected_controllers(input: &Input) -> Vec<u64> {
    input
        .get_connected_controllers()
        .into_iter()
        .filter(|&c| c != 0)
        .collect()
}

/// Action-set names — MUST match the IGA manifest
/// (`assets/controller_config/game_actions_4550880.vdf`).
pub(crate) const GAMEPLAY_SET: &str = "GameplayControls";
pub(crate) const MENU_SET: &str = "MenuControls";

/// Resolved Steam Input handles for our action sets and actions.
#[derive(Resource, Default)]
pub(crate) struct SteamInputHandles {
    pub gameplay_set: u64,
    pub menu_set: u64,
    pub digital: HashMap<GamepadAction, u64>,
    pub analog: HashMap<AnalogAction, u64>,
    /// True once every handle resolved to non-zero (the manifest has loaded).
    pub resolved: bool,
}

impl SteamInputHandles {
    /// All action-set + action handles are non-zero.
    fn all_present(&self) -> bool {
        self.gameplay_set != 0
            && self.menu_set != 0
            && GamepadAction::ALL
                .iter()
                .all(|a| self.digital.get(a).copied().unwrap_or(0) != 0)
            && AnalogAction::ALL
                .iter()
                .all(|a| self.analog.get(a).copied().unwrap_or(0) != 0)
    }
}

/// Resolves the action-set + action handles from the loaded manifest. Handles
/// read 0 until Steam has loaded the IGA (which needs `run_frame` to have run and
/// a controller to be present), so this retries every frame until everything is
/// non-zero, then marks itself resolved and stops doing FFI work.
pub(crate) fn resolve_steam_input_handles(
    client: Res<Client>,
    mut handles: ResMut<SteamInputHandles>,
) {
    if handles.resolved {
        return;
    }
    let input = client.input();
    handles.gameplay_set = input.get_action_set_handle(GAMEPLAY_SET);
    handles.menu_set = input.get_action_set_handle(MENU_SET);
    for action in GamepadAction::ALL {
        let h = input.get_digital_action_handle(action.manifest_name());
        handles.digital.insert(action, h);
    }
    for action in AnalogAction::ALL {
        let h = input.get_analog_action_handle(action.manifest_name());
        handles.analog.insert(action, h);
    }
    if handles.all_present() {
        handles.resolved = true;
        info!("[Steam Input] action manifest loaded; all handles resolved");
    }
}
