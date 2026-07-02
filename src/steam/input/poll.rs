//! Steam Input poll producer — the primary controller reader.
//!
//! Each frame (after `run_frame` + handle resolution + action-set activation),
//! reads the active controller's action data into [`GamepadActionState`] and
//! claims [`ActiveInputDevice::SteamInputPad`] on activity. The gilrs adapter,
//! which runs right after, sees `SteamInputPad` and yields, so exactly one
//! producer fills the state per frame.

use bevy::prelude::*;
use bevy_steamworks::Client;
use steamworks::InputType;

use super::handles::{SteamInputHandles, connected_controllers};
use crate::config::GameConfig;
use crate::game::input::action_state::{
    AnalogAction, ControllerKind, GamepadAction, GamepadActionState,
};
use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::game::multiplayer::pause_request::RequestGamePauseMessage;

/// Stick deflection that counts as activity for seizing the active device
/// (mirrors the gilrs device-switch threshold).
const ACTIVITY_STICK: f32 = 0.25;
/// Trigger pull that counts as activity.
const ACTIVITY_TRIGGER: f32 = 0.5;

fn kind_from_input_type(t: InputType) -> ControllerKind {
    match t {
        InputType::PS3Controller | InputType::PS4Controller | InputType::PS5Controller => {
            ControllerKind::PlayStation
        }
        InputType::SwitchJoyConPair
        | InputType::SwitchJoyConSingle
        | InputType::SwitchProController => ControllerKind::Switch,
        InputType::SteamDeckController | InputType::SteamController => ControllerKind::SteamDeck,
        _ => ControllerKind::Xbox,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn poll_steam_input(
    client: Res<Client>,
    handles: Res<SteamInputHandles>,
    config: Res<GameConfig>,
    mut state: ResMut<GamepadActionState>,
    mut active: ResMut<ActiveInputDevice>,
    mut pause_writer: MessageWriter<RequestGamePauseMessage>,
    mut last_count: Local<Option<usize>>,
) {
    if !handles.resolved {
        return;
    }
    let input = client.input();
    let controllers = connected_controllers(&input);

    // Log whenever Steam's view of the connected-controller count changes — a
    // controller power-off should print "0 controllers"; if it doesn't, Steam
    // itself didn't notice the disconnect.
    if *last_count != Some(controllers.len()) {
        info!("[Steam Input] connected controllers: {}", controllers.len());
        *last_count = Some(controllers.len());
    }

    // Disconnect of the controller we were driving: reset to mouse/keyboard and
    // (if configured) request the auto-pause. gilrs's unplug handler never fires
    // for a Steam-captured pad, so the Steam path must do this itself.
    if let Some(h) = state.active_steam_handle
        && !controllers.contains(&h)
    {
        info!(
            "[Steam Input] active controller disconnected; reset + pause (enabled={})",
            config.pause_on_controller_disconnect
        );
        state.clear_inputs();
        if matches!(*active, ActiveInputDevice::SteamInputPad) {
            *active = ActiveInputDevice::MouseKeyboard;
        }
        if config.pause_on_controller_disconnect {
            pause_writer.write(RequestGamePauseMessage);
        }
    }

    if controllers.is_empty() {
        return;
    }

    // Stay on the controller we were already driving if it's still connected.
    let candidate = state
        .active_steam_handle
        .filter(|h| controllers.contains(h))
        .unwrap_or(controllers[0]);

    // Read the candidate's action data (the active set's bindings; actions not in
    // the active set report bState=false / value 0).
    let mut digital = [false; GamepadAction::ALL.len()];
    for (i, action) in GamepadAction::ALL.iter().enumerate() {
        if let Some(&h) = handles.digital.get(action) {
            digital[i] = input.get_digital_action_data(candidate, h).bState;
        }
    }
    let analog = |a: AnalogAction| -> (f32, f32) {
        handles
            .analog
            .get(&a)
            .map(|&h| {
                let d = input.get_analog_action_data(candidate, h);
                (d.x, d.y)
            })
            .unwrap_or((0.0, 0.0))
    };
    let (lx, ly) = analog(AnalogAction::AimStick);
    let (rx, ry) = analog(AnalogAction::Camera);
    // Analog triggers report their value in `x` (study-tab zoom). Only bound in
    // MenuControls; in gameplay the triggers drive digital PrimaryCast/SecondaryCast.
    let lt = analog(AnalogAction::ZoomOut).0;
    let rt = analog(AnalogAction::ZoomIn).0;

    let any_digital = digital.iter().any(|&b| b);
    let stick_active =
        Vec2::new(lx, ly).length() > ACTIVITY_STICK || Vec2::new(rx, ry).length() > ACTIVITY_STICK;
    let trigger_active = lt > ACTIVITY_TRIGGER || rt > ACTIVITY_TRIGGER;
    let active_now = any_digital || stick_active || trigger_active;

    let already_steam = matches!(*active, ActiveInputDevice::SteamInputPad)
        && state.active_steam_handle == Some(candidate);

    // Only seize the active device on fresh activity (so a resting controller
    // doesn't hijack mouse/keyboard); once we own it, keep driving.
    if !active_now && !already_steam {
        return;
    }

    // Only write when it actually changes — an unconditional assignment marks
    // `ActiveInputDevice` changed every frame, which retriggers
    // `resource_changed::<ActiveInputDevice>` consumers (e.g. the action-bar
    // reset that wipes the radial hover highlight) every frame.
    if *active != ActiveInputDevice::SteamInputPad {
        *active = ActiveInputDevice::SteamInputPad;
    }
    state.active_steam_handle = Some(candidate);
    state.active_gilrs_entity = None;
    state.controller_kind = kind_from_input_type(input.get_input_type_for_handle(candidate));
    // Steam joystick_move reports +Y up, matching gilrs and our consumers' raw
    // expectation (shape_stick flips Y itself). Validate on hardware; flip here if
    // aiming is inverted.
    state.left_stick = Vec2::new(lx, ly);
    state.right_stick = Vec2::new(rx, ry);
    state.left_trigger = lt;
    state.right_trigger = rt;

    state.buttons.clear();
    for (i, action) in GamepadAction::ALL.iter().enumerate() {
        if digital[i] {
            state.buttons.press(*action);
        } else {
            state.buttons.release(*action);
        }
    }
}
