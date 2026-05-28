//! Join-code text input: focused keyboard capture + clipboard paste.

use bevy::prelude::*;

use super::state::{JoinCodeInputBox, JoinCodeInputDisplay, MultiplayerLobby};

/// Handles keyboard input when the join-code field is focused.
pub(crate) fn handle_join_code_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut lobby: ResMut<MultiplayerLobby>,
    mut text_query: Query<&mut Text, With<JoinCodeInputDisplay>>,
    mut border_query: Query<&mut BorderColor, With<JoinCodeInputBox>>,
) {
    if lobby.is_changed() {
        let border = if lobby.join_code_focused {
            BorderColor::all(Color::hsla(270.0, 0.65, 0.55, 1.0))
        } else {
            BorderColor::all(Color::hsla(270.0, 0.35, 0.35, 1.0))
        };
        for mut b in &mut border_query {
            *b = border;
        }
    }

    if !lobby.join_code_focused {
        return;
    }

    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let mut changed = false;

    if ctrl
        && keyboard.just_pressed(KeyCode::KeyV)
        && let Ok(text) = arboard::Clipboard::new().and_then(|mut cb| cb.get_text())
    {
        let trimmed = text.trim().to_string();
        if !trimmed.is_empty() {
            lobby.join_code_input = trimmed;
            lobby.join_code_focused = false;
            changed = true;
        }
    }

    if keyboard.just_pressed(KeyCode::Backspace) && !lobby.join_code_input.is_empty() {
        lobby.join_code_input.pop();
        changed = true;
    }

    if !ctrl {
        for &(key, lower, upper) in printable_keys() {
            if keyboard.just_pressed(key) {
                let shift =
                    keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
                let ch = if shift { upper } else { lower };
                lobby.join_code_input.push(ch);
                changed = true;
            }
        }
    }

    if changed {
        for mut text in &mut text_query {
            text.0 = if lobby.join_code_input.is_empty() {
                "Click to type or paste code...".to_string()
            } else {
                lobby.join_code_input.clone()
            };
        }
    }
}

fn printable_keys() -> &'static [(KeyCode, char, char)] {
    &[
        (KeyCode::KeyA, 'a', 'A'),
        (KeyCode::KeyB, 'b', 'B'),
        (KeyCode::KeyC, 'c', 'C'),
        (KeyCode::KeyD, 'd', 'D'),
        (KeyCode::KeyE, 'e', 'E'),
        (KeyCode::KeyF, 'f', 'F'),
        (KeyCode::KeyG, 'g', 'G'),
        (KeyCode::KeyH, 'h', 'H'),
        (KeyCode::KeyI, 'i', 'I'),
        (KeyCode::KeyJ, 'j', 'J'),
        (KeyCode::KeyK, 'k', 'K'),
        (KeyCode::KeyL, 'l', 'L'),
        (KeyCode::KeyM, 'm', 'M'),
        (KeyCode::KeyN, 'n', 'N'),
        (KeyCode::KeyO, 'o', 'O'),
        (KeyCode::KeyP, 'p', 'P'),
        (KeyCode::KeyQ, 'q', 'Q'),
        (KeyCode::KeyR, 'r', 'R'),
        (KeyCode::KeyS, 's', 'S'),
        (KeyCode::KeyT, 't', 'T'),
        (KeyCode::KeyU, 'u', 'U'),
        (KeyCode::KeyV, 'v', 'V'),
        (KeyCode::KeyW, 'w', 'W'),
        (KeyCode::KeyX, 'x', 'X'),
        (KeyCode::KeyY, 'y', 'Y'),
        (KeyCode::KeyZ, 'z', 'Z'),
        (KeyCode::Digit0, '0', ')'),
        (KeyCode::Digit1, '1', '!'),
        (KeyCode::Digit2, '2', '@'),
        (KeyCode::Digit3, '3', '#'),
        (KeyCode::Digit4, '4', '$'),
        (KeyCode::Digit5, '5', '%'),
        (KeyCode::Digit6, '6', '^'),
        (KeyCode::Digit7, '7', '&'),
        (KeyCode::Digit8, '8', '*'),
        (KeyCode::Digit9, '9', '('),
        (KeyCode::Equal, '=', '+'),
        (KeyCode::Minus, '-', '_'),
    ]
}
