use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::MouseButtonState;
use crate::game::input::gamepad::messages::MenuBackPressed;
use crate::state::{InGameState, MenuState, MultiplayerGameState, PauseMenuState};

/// Generic "Back" button-click handler returning to the main-menu landing screen.
/// Parameterized by the screen's own back-button marker component `B` so each
/// screen (compendium, manual, …) reuses one implementation.
pub(crate) fn handle_main_menu_back_button<B: Component>(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    button_query: Query<&B>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            channel_change.write(ChannelChangeMessage);
            next_state.set(MenuState::Landing);
        }
    }
}

/// Generic "Back" button-click handler returning to the pause-menu main screen.
pub(crate) fn handle_pause_menu_back_button<B: Component>(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    button_query: Query<&B>,
    mut next_state: ResMut<NextState<PauseMenuState>>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            next_state.set(PauseMenuState::Main);
        }
    }
}

/// Handles Escape key / gamepad back to return to the main menu landing screen.
pub fn escape_to_landing(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut back_msgs: MessageReader<MenuBackPressed>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    let back_pressed = back_msgs.read().next().is_some();
    if keyboard.just_pressed(KeyCode::Escape) || back_pressed {
        channel_change.write(ChannelChangeMessage);
        next_state.set(MenuState::Landing);
    }
}

/// Handles Escape key / gamepad back to return to the pause menu main screen.
pub fn escape_to_pause_main(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut back_msgs: MessageReader<MenuBackPressed>,
    mut next_state: ResMut<NextState<PauseMenuState>>,
) {
    let back_pressed = back_msgs.read().next().is_some();
    if keyboard.just_pressed(KeyCode::Escape) || back_pressed {
        next_state.set(PauseMenuState::Main);
    }
}

/// Handles Escape / gamepad East or Start to return to running gameplay state.
///
/// Reads gamepad buttons directly via `just_pressed` rather than the
/// `MenuBackPressed` message — the latter persists for an extra frame,
/// which would re-fire immediately after entering a paused state and flip
/// back to Running (Start-in-Running → pause → Paused frame sees leftover
/// back-press → unpause).
pub fn escape_to_running(
    keyboard: Res<ButtonInput<KeyCode>>,
    active: Res<crate::game::input::gamepad::resources::ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    mut next_in_game_state: Option<ResMut<NextState<InGameState>>>,
    mut next_mp_state: Option<ResMut<NextState<MultiplayerGameState>>>,
) {
    let gamepad_back = active
        .gamepad_entity()
        .and_then(|e| gamepads.get(e).ok())
        .is_some_and(|g| {
            g.just_pressed(GamepadButton::East) || g.just_pressed(GamepadButton::Start)
        });
    if keyboard.just_pressed(KeyCode::Escape) || gamepad_back {
        if let Some(ref mut next_sp) = next_in_game_state {
            next_sp.set(InGameState::Running);
        }
        if let Some(ref mut next_mp) = next_mp_state {
            next_mp.set(MultiplayerGameState::Running);
        }
    }
}

/// Consumes the mouse button state on exit to prevent click bleed-through.
pub fn consume_mouse_on_exit(mut mouse_state: ResMut<MouseButtonState>) {
    mouse_state.left_consumed = true;
}
