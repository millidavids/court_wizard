use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::resources::{ActiveInputDevice, GamepadAimSettings, VirtualCursorPosition};
use crate::game::crt_effect::CorrectedCursorPosition;

/// Integrates the left stick into a screen-space virtual cursor and overwrites
/// `CorrectedCursorPosition` during gameplay.
///
/// In menu contexts (`InGameState != Running`, or outside `InGame`), the
/// cursor is parked (`None`) so stick motion drives only the focus system and
/// doesn't incidentally hover buttons via `correct_ui_interaction_for_barrel`.
///
/// Runs in PreUpdate after `correct_cursor_for_barrel_distortion`, so the
/// barrel corrector's mouse-mode output remains authoritative while gamepad
/// gameplay overrides with the virtual cursor.
#[allow(clippy::too_many_arguments)]
pub(crate) fn update_virtual_cursor(
    time: Res<Time>,
    active: Res<ActiveInputDevice>,
    aim: Res<GamepadAimSettings>,
    gamepads: Query<&Gamepad>,
    windows: Query<&Window, With<PrimaryWindow>>,
    in_game: Option<Res<State<crate::state::InGameState>>>,
    mp_game: Option<Res<State<crate::state::MultiplayerGameState>>>,
    mut virtual_cursor: ResMut<VirtualCursorPosition>,
    mut corrected: ResMut<CorrectedCursorPosition>,
) {
    let Some(gamepad_entity) = active.gamepad_entity() else {
        return;
    };

    // Active gameplay in single-player OR multiplayer (in MP, `InGameState`
    // doesn't exist — `MultiplayerGameState` does — so without this the virtual
    // cursor would be forced to None and gamepad casts couldn't aim.)
    let sp_gameplay = in_game
        .as_deref()
        .map(|s| *s.get() == crate::state::InGameState::Running)
        .unwrap_or(false);
    let mp_gameplay = mp_game
        .as_deref()
        .map(|s| *s.get() == crate::state::MultiplayerGameState::Running)
        .unwrap_or(false);
    if !sp_gameplay && !mp_gameplay {
        corrected.0 = None;
        return;
    }

    let Ok(gamepad) = gamepads.get(gamepad_entity) else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };

    let delta = read_left_stick_shaped(gamepad, &aim);

    let dt = time.delta_secs();
    virtual_cursor.screen_pos.x += delta.x * aim.sensitivity.x * dt;
    virtual_cursor.screen_pos.y += delta.y * aim.sensitivity.y * dt;

    // Clamp to window bounds so the cursor can't leave the viewport.
    virtual_cursor.screen_pos.x = virtual_cursor.screen_pos.x.clamp(0.0, window.width());
    virtual_cursor.screen_pos.y = virtual_cursor.screen_pos.y.clamp(0.0, window.height());

    corrected.0 = Some(virtual_cursor.screen_pos);
}

/// Applies a radial deadzone and response curve to a stick input vector.
///
/// Returns a vector with magnitude rescaled from `[deadzone, 1.0]` → `[0.0, 1.0]`
/// then raised to `curve` (e.g. 2.2 = ease-out). Keeps the original direction.
/// Reads the left-stick axes, flips Y so "stick up" maps to screen-up (-Y),
/// and routes through `apply_deadzone_and_curve`. Shared by gameplay's
/// virtual cursor and the Study tab's spell-web cursor/edge-scroll.
pub(crate) fn read_left_stick_shaped(gamepad: &Gamepad, aim: &GamepadAimSettings) -> Vec2 {
    let lx = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
    let ly = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
    apply_deadzone_and_curve(Vec2::new(lx, -ly), aim.deadzone, aim.response_curve)
}

pub(crate) fn apply_deadzone_and_curve(input: Vec2, deadzone: f32, curve: f32) -> Vec2 {
    let mag = input.length();
    if mag < deadzone {
        return Vec2::ZERO;
    }
    let normalized = (mag - deadzone) / (1.0 - deadzone);
    let shaped = normalized.clamp(0.0, 1.0).powf(curve);
    input.normalize_or_zero() * shaped
}
