use bevy::input::gamepad::GamepadConnectionEvent;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::window::{CursorMoved, CursorOptions, PrimaryWindow};

use super::super::resources::{ActiveInputDevice, VirtualCursorPosition};
use crate::config::GameConfig;
use crate::game::input::components::MouseButtonState;
use crate::game::multiplayer::pause_request::RequestGamePauseMessage;

use super::super::constants::DEVICE_SWITCH_STICK_MAGNITUDE;

/// Updates `ActiveInputDevice` each frame based on which input source was used.
///
/// Any mouse/keyboard activity → `MouseKeyboard`.
/// Any gamepad activity (stick deflection past hysteresis, or button press) → `Gamepad(entity)`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn detect_active_input_device(
    mut active: ResMut<ActiveInputDevice>,
    mut cursor_moved: MessageReader<CursorMoved>,
    mut mouse_buttons: MessageReader<MouseButtonInput>,
    mut keyboard: MessageReader<KeyboardInput>,
    gamepads: Query<(Entity, &Gamepad)>,
) {
    // Mouse / keyboard events → switch to mouse/keyboard.
    let mk_activity = cursor_moved.read().next().is_some()
        || mouse_buttons.read().next().is_some()
        || keyboard.read().next().is_some();
    if mk_activity && !matches!(*active, ActiveInputDevice::MouseKeyboard) {
        *active = ActiveInputDevice::MouseKeyboard;
        return;
    }

    // Gamepad activity → switch to gamepad.
    for (entity, gamepad) in &gamepads {
        let any_button =
            gamepad.get_just_pressed().next().is_some() || gamepad.get_pressed().next().is_some();
        let mag_sq = [
            GamepadAxis::LeftStickX,
            GamepadAxis::LeftStickY,
            GamepadAxis::RightStickX,
            GamepadAxis::RightStickY,
        ]
        .iter()
        .map(|ax| gamepad.get(*ax).unwrap_or(0.0))
        .fold(0.0f32, |acc, v| acc + v * v);

        let stick_active = mag_sq > DEVICE_SWITCH_STICK_MAGNITUDE * DEVICE_SWITCH_STICK_MAGNITUDE;

        if any_button || stick_active {
            let should_switch = match *active {
                ActiveInputDevice::Gamepad(e) => e != entity,
                ActiveInputDevice::MouseKeyboard | ActiveInputDevice::SteamInputPad => true,
            };
            if should_switch {
                *active = ActiveInputDevice::Gamepad(entity);
            }
            return;
        }
    }
}

/// Hides the OS cursor when gamepad becomes active, shows it when mouse takes over.
/// Also clears stale mouse state on transition so a lingering left-click doesn't
/// accidentally fire a spell after switching to controller.
pub(crate) fn toggle_cursor_visibility(
    active: Res<ActiveInputDevice>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut last: Local<ActiveInputDevice>,
    mut virtual_cursor: ResMut<VirtualCursorPosition>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut mouse_state: ResMut<MouseButtonState>,
) {
    if *last == *active {
        return;
    }
    *last = *active;

    let Ok(mut cursor) = cursor_options.single_mut() else {
        return;
    };

    match *active {
        ActiveInputDevice::Gamepad(_) | ActiveInputDevice::SteamInputPad => {
            cursor.visible = false;
            // Start the virtual cursor at screen center so the player doesn't have to hunt for it.
            if let Ok(window) = windows.single() {
                virtual_cursor.screen_pos = Vec2::new(window.width() * 0.5, window.height() * 0.5);
            }
        }
        ActiveInputDevice::MouseKeyboard => {
            cursor.visible = true;
        }
    }

    mouse.clear();
    mouse_state.left_consumed = false;
}

/// Pauses the game when the *active* controller disconnects mid-match, and falls
/// back to mouse/keyboard so the pause menu stays usable.
///
/// On disconnect Bevy removes the `Gamepad` component (leaving the entity alive),
/// so every gamepad reader would early-return on the now-dead handle and the OS
/// cursor would stay hidden. Resetting `ActiveInputDevice` is unconditional
/// robustness; only the pause itself is behind the config flag, routed through
/// the shared `RequestGamePauseMessage` consumer so co-op pauses both peers.
pub(crate) fn pause_on_controller_unplug(
    mut events: MessageReader<GamepadConnectionEvent>,
    mut active: ResMut<ActiveInputDevice>,
    config: Res<GameConfig>,
    mut pause_writer: MessageWriter<RequestGamePauseMessage>,
) {
    let active_entity = active.gamepad_entity();
    let mut active_lost = false;
    for event in events.read() {
        if event.disconnected() && Some(event.gamepad) == active_entity {
            active_lost = true;
        }
    }
    if active_lost {
        *active = ActiveInputDevice::MouseKeyboard;
        if config.pause_on_controller_disconnect {
            pause_writer.write(RequestGamePauseMessage);
        }
    }
}
