//! Steam Input haptics (rumble) and the binding-panel launcher.
//!
//! The safe `Input` wrapper exposes neither `TriggerVibration` nor the raw
//! interface pointer (`pub(crate)`), so haptics go through the `steamworks-sys`
//! flat accessor. The binding panel is available on the safe wrapper.

use bevy::prelude::*;
use bevy_steamworks::Client;

use crate::game::input::action_state::GamepadActionState;
use crate::game::input::gamepad::messages::ControllerRumbleMessage;
use crate::game::input::messages::MouseClicked;
use crate::ui::main_menu::settings::components::ConfigureControllerButton;

/// Sets the vibration motors on a Steam Input controller. `left`/`right` are motor
/// speeds, 0..=65535. No-op if Steam Input isn't initialized (null interface).
fn trigger_vibration(handle: u64, left: u16, right: u16) {
    // SAFETY: the flat accessor returns the process-wide Steam Input interface
    // (initialized by `init_steam_input`); `TriggerVibration` is documented as
    // safe to call with any handle. We null-check the interface defensively.
    unsafe {
        let iface = steamworks_sys::SteamAPI_SteamInput_v006();
        if !iface.is_null() {
            steamworks_sys::SteamAPI_ISteamInput_TriggerVibration(iface, handle, left, right);
        }
    }
}

/// Schedules the stop of an in-flight rumble pulse (TriggerVibration sets a
/// sustained speed; we zero it when the pulse's duration elapses).
#[derive(Resource)]
pub(crate) struct SteamRumbleStop {
    handle: u64,
    timer: Timer,
}

fn to_motor(intensity: f32) -> u16 {
    (intensity.clamp(0.0, 1.0) * u16::MAX as f32) as u16
}

/// Starts a Steam Input rumble pulse for each [`ControllerRumbleMessage`] and
/// arms its stop timer. Only the most recent request matters at the score screen.
pub(crate) fn apply_steam_rumble(
    mut events: MessageReader<ControllerRumbleMessage>,
    state: Res<GamepadActionState>,
    mut commands: Commands,
) {
    let Some(msg) = events.read().last() else {
        return;
    };
    let Some(handle) = state.active_steam_handle else {
        return;
    };
    trigger_vibration(handle, to_motor(msg.strong), to_motor(msg.weak));
    commands.insert_resource(SteamRumbleStop {
        handle,
        timer: Timer::new(msg.duration, TimerMode::Once),
    });
}

/// Zeros the motors when the active rumble pulse's duration elapses.
pub(crate) fn tick_steam_rumble_stop(
    time: Res<Time>,
    stop: Option<ResMut<SteamRumbleStop>>,
    mut commands: Commands,
) {
    let Some(mut stop) = stop else {
        return;
    };
    if stop.timer.tick(time.delta()).just_finished() {
        trigger_vibration(stop.handle, 0, 0);
        commands.remove_resource::<SteamRumbleStop>();
    }
}

/// Opens Steam's controller-configuration (binding) panel when the
/// "Configure Controller" settings button is clicked, for the active Steam pad.
pub(crate) fn open_binding_panel_on_click(
    mut clicks: MessageReader<MouseClicked>,
    buttons: Query<(), With<ConfigureControllerButton>>,
    client: Res<Client>,
    state: Res<GamepadActionState>,
) {
    let clicked_button = clicks.read().any(|e| buttons.contains(e.button));
    if !clicked_button {
        return;
    }
    let Some(handle) = state.active_steam_handle else {
        info!("[Steam Input] Configure Controller pressed but no Steam Input pad is active");
        return;
    };
    let opened = client.input().show_binding_panel(handle);
    info!("[Steam Input] binding panel opened: {opened}");
}
