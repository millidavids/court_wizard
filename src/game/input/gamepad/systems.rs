//! Gamepad input systems.
//!
//! Translates raw gamepad axes and buttons into the same messages that the
//! existing keyboard/mouse pipeline uses. Downstream gameplay and UI systems
//! are unaware of the source — they read `CorrectedCursorPosition`,
//! `MouseLeftPressed/Held/Released`, `MouseRightPressed/Held/Released`, and
//! `MouseClicked` as before.

use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::window::{CursorMoved, CursorOptions, PrimaryWindow};

use super::constants::{DEVICE_SWITCH_STICK_MAGNITUDE, RADIAL_SLOT_COUNT};
use super::messages::{GamepadConfirmPressed, MenuBackPressed};
use super::resources::{
    ActiveInputDevice, GamepadAimSettings, RadialHoveredSlot, VirtualCursorPosition,
};
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::components::MouseButtonState;
use crate::game::input::messages::{
    MouseLeftHeld, MouseLeftPressed, MouseLeftReleased, MouseRightHeld, MouseRightPressed,
    MouseRightReleased,
};

// ---------------------------------------------------------------------------
// Device source detection
// ---------------------------------------------------------------------------

/// Updates `ActiveInputDevice` each frame based on which input source was used.
///
/// Any mouse/keyboard activity → `MouseKeyboard`.
/// Any gamepad activity (stick deflection past hysteresis, or button press) → `Gamepad(entity)`.
#[allow(clippy::too_many_arguments)]
pub(super) fn detect_active_input_device(
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

        let stick_active = mag_sq.sqrt() > DEVICE_SWITCH_STICK_MAGNITUDE;

        if any_button || stick_active {
            let should_switch = match *active {
                ActiveInputDevice::Gamepad(e) => e != entity,
                ActiveInputDevice::MouseKeyboard => true,
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
pub(super) fn toggle_cursor_visibility(
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
        ActiveInputDevice::Gamepad(_) => {
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

// ---------------------------------------------------------------------------
// Virtual cursor (left stick)
// ---------------------------------------------------------------------------

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
pub(super) fn update_virtual_cursor(
    time: Res<Time>,
    active: Res<ActiveInputDevice>,
    aim: Res<GamepadAimSettings>,
    gamepads: Query<&Gamepad>,
    windows: Query<&Window, With<PrimaryWindow>>,
    in_game: Option<Res<State<crate::state::InGameState>>>,
    mut virtual_cursor: ResMut<VirtualCursorPosition>,
    mut corrected: ResMut<CorrectedCursorPosition>,
) {
    let Some(gamepad_entity) = active.gamepad_entity() else {
        return;
    };

    let is_gameplay = in_game
        .as_deref()
        .map(|s| *s.get() == crate::state::InGameState::Running)
        .unwrap_or(false);
    if !is_gameplay {
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

// ---------------------------------------------------------------------------
// Trigger → mouse message translation (with radial commit intercept)
// ---------------------------------------------------------------------------

/// Translates right-trigger / left-trigger into the mouse-button messages
/// every spell already consumes.
///
/// Radial commit intercept: when RT is pressed while the right stick is
/// deflected past the deadzone, the press is interpreted as committing the
/// hovered radial action bar slot — `MouseLeftPressed/Held/Released` are
/// suppressed for the lifetime of that RT hold, and the hovered slot is
/// emitted as `ActionBarKeyPressed`.
///
/// Custom trigger threshold comes from `GamepadAimSettings`, so we can't use
/// Bevy's built-in digital `just_pressed`/`just_released` on these buttons —
/// hence the `Local` edge tracking.
#[allow(clippy::too_many_arguments)]
pub(super) fn translate_triggers_to_mouse_messages(
    active: Res<ActiveInputDevice>,
    aim: Res<GamepadAimSettings>,
    gamepads: Query<&Gamepad>,
    hovered_slot: Res<RadialHoveredSlot>,
    virtual_cursor: Res<VirtualCursorPosition>,
    mut prev_triggers: Local<(bool, bool)>,
    mut commit_armed: Local<bool>,
    mut action_bar: MessageWriter<crate::game::input::messages::ActionBarKeyPressed>,
    mut left_pressed: MessageWriter<MouseLeftPressed>,
    mut left_held: MessageWriter<MouseLeftHeld>,
    mut left_released: MessageWriter<MouseLeftReleased>,
    mut right_pressed: MessageWriter<MouseRightPressed>,
    mut right_held: MessageWriter<MouseRightHeld>,
    mut right_released: MessageWriter<MouseRightReleased>,
) {
    let Some(gamepad_entity) = active.gamepad_entity() else {
        *prev_triggers = (false, false);
        *commit_armed = false;
        return;
    };
    let Ok(gamepad) = gamepads.get(gamepad_entity) else {
        return;
    };

    let rt_now = gamepad.get(GamepadButton::RightTrigger2).unwrap_or(0.0) >= aim.trigger_threshold;
    let lt_now = gamepad.get(GamepadButton::LeftTrigger2).unwrap_or(0.0) >= aim.trigger_threshold;
    let (rt_prev, lt_prev) = *prev_triggers;
    *prev_triggers = (rt_now, lt_now);

    let cursor_position = Some(virtual_cursor.screen_pos);

    match (rt_prev, rt_now) {
        (false, true) => {
            if let Some(slot) = hovered_slot.0 {
                action_bar.write(crate::game::input::messages::ActionBarKeyPressed { slot });
                *commit_armed = true;
            } else {
                left_pressed.write(MouseLeftPressed { cursor_position });
                left_held.write(MouseLeftHeld { cursor_position });
            }
        }
        (true, true) => {
            if !*commit_armed {
                left_held.write(MouseLeftHeld { cursor_position });
            }
        }
        (true, false) => {
            if *commit_armed {
                *commit_armed = false;
            } else {
                left_released.write(MouseLeftReleased);
            }
        }
        (false, false) => {}
    }

    match (lt_prev, lt_now) {
        (false, true) => {
            right_pressed.write(MouseRightPressed { cursor_position });
            right_held.write(MouseRightHeld { cursor_position });
        }
        (true, true) => {
            right_held.write(MouseRightHeld { cursor_position });
        }
        (true, false) => {
            right_released.write(MouseRightReleased);
        }
        (false, false) => {}
    }
}

// ---------------------------------------------------------------------------
// Right-stick → radial slot
// ---------------------------------------------------------------------------

/// Maps right-stick deflection to a radial action bar slot index (0..=4).
///
/// 0° = straight up = slot 0, clockwise. Each slot owns a 72° wedge centered
/// on its ideal angle. Returns `None` when the stick is in the deadzone.
pub(crate) fn right_stick_to_slot(stick: Vec2, deadzone: f32) -> Option<u8> {
    if stick.length_squared() < deadzone * deadzone {
        return None;
    }
    // x.atan2(y) gives angle measured clockwise from "up".
    let angle_deg = stick.x.atan2(stick.y).to_degrees();
    let angle_norm = (angle_deg + 360.0).rem_euclid(360.0);
    // Center each wedge on its ideal angle by offsetting by half a wedge width.
    let wedge = super::constants::RADIAL_WEDGE_DEGREES;
    let slot = (((angle_norm + wedge * 0.5) / wedge) as u32) % RADIAL_SLOT_COUNT as u32;
    Some(slot as u8)
}

/// Reads the right stick each frame and updates `RadialHoveredSlot`.
pub(super) fn update_radial_hovered_slot(
    active: Res<ActiveInputDevice>,
    aim: Res<GamepadAimSettings>,
    gamepads: Query<&Gamepad>,
    mut hovered: ResMut<RadialHoveredSlot>,
) {
    let Some(gamepad_entity) = active.gamepad_entity() else {
        if hovered.0.is_some() {
            hovered.0 = None;
        }
        return;
    };
    let Ok(gamepad) = gamepads.get(gamepad_entity) else {
        return;
    };

    let stick = Vec2::new(
        gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0),
        gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0),
    );
    let new_slot = right_stick_to_slot(stick, aim.deadzone);
    if new_slot != hovered.0 {
        hovered.0 = new_slot;
    }
}

// ---------------------------------------------------------------------------
// UI confirm / back
// ---------------------------------------------------------------------------

/// Syncs the user's `GameConfig` gamepad settings into `GamepadAimSettings`.
///
/// `GameConfig` is mutated by many unrelated systems (save data, level progress).
/// We only rewrite `GamepadAimSettings` when our four tuning fields actually change
/// to avoid cascading `Changed<GamepadAimSettings>` triggers.
pub(super) fn sync_gamepad_settings(
    config: Res<crate::config::GameConfig>,
    mut aim: ResMut<GamepadAimSettings>,
) {
    let new_sensitivity = Vec2::new(
        super::constants::DEFAULT_SENSITIVITY_X * config.gamepad_sensitivity_x,
        super::constants::DEFAULT_SENSITIVITY_Y * config.gamepad_sensitivity_y,
    );
    if aim.deadzone == config.gamepad_deadzone
        && aim.response_curve == config.gamepad_response_curve
        && aim.sensitivity == new_sensitivity
    {
        return;
    }
    aim.deadzone = config.gamepad_deadzone;
    aim.response_curve = config.gamepad_response_curve;
    aim.sensitivity = new_sensitivity;
}

/// Emits `GamepadConfirmPressed` on South (A/Cross) and `MenuBackPressed` on
/// East (B/Circle) or Start. Downstream focus-navigation and escape-handlers
/// consume these.
pub(super) fn emit_ui_confirm_back_messages(
    active: Res<ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    mut confirm: MessageWriter<GamepadConfirmPressed>,
    mut back: MessageWriter<MenuBackPressed>,
) {
    let Some(gamepad_entity) = active.gamepad_entity() else {
        return;
    };
    let Ok(gamepad) = gamepads.get(gamepad_entity) else {
        return;
    };
    if gamepad.just_pressed(GamepadButton::South) {
        confirm.write(GamepadConfirmPressed);
    }
    if gamepad.just_pressed(GamepadButton::East) || gamepad.just_pressed(GamepadButton::Start) {
        back.write(MenuBackPressed);
    }
}
