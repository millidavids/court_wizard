use bevy::prelude::*;

use super::super::constants::{
    DEFAULT_SENSITIVITY_X, DEFAULT_SENSITIVITY_Y, RADIAL_SLOT_COUNT, RADIAL_WEDGE_DEGREES,
};
use super::super::messages::{GamepadConfirmPressed, MenuBackPressed};
use super::super::resources::{
    ActiveInputDevice, GamepadAimSettings, RadialHoveredSlot, VirtualCursorPosition,
};
use crate::game::input::messages::{
    ActionBarKeyPressed, MouseLeftHeld, MouseLeftPressed, MouseLeftReleased, MouseRightHeld,
    MouseRightPressed, MouseRightReleased,
};

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
pub(crate) fn translate_triggers_to_mouse_messages(
    active: Res<ActiveInputDevice>,
    aim: Res<GamepadAimSettings>,
    gamepads: Query<&Gamepad>,
    hovered_slot: Res<RadialHoveredSlot>,
    virtual_cursor: Res<VirtualCursorPosition>,
    mut prev_triggers: Local<(bool, bool)>,
    mut commit_armed: Local<bool>,
    mut action_bar: MessageWriter<ActionBarKeyPressed>,
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
                action_bar.write(ActionBarKeyPressed { slot });
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
    let wedge = RADIAL_WEDGE_DEGREES;
    let slot = (((angle_norm + wedge * 0.5) / wedge) as u32) % RADIAL_SLOT_COUNT as u32;
    Some(slot as u8)
}

/// Reads the right stick each frame and updates `RadialHoveredSlot`.
pub(crate) fn update_radial_hovered_slot(
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

/// Syncs the user's `GameConfig` gamepad settings into `GamepadAimSettings`.
///
/// `GameConfig` is mutated by many unrelated systems (save data, level progress).
/// We only rewrite `GamepadAimSettings` when our four tuning fields actually change
/// to avoid cascading `Changed<GamepadAimSettings>` triggers.
pub(crate) fn sync_gamepad_settings(
    config: Res<crate::config::GameConfig>,
    mut aim: ResMut<GamepadAimSettings>,
) {
    let new_sensitivity = Vec2::new(
        DEFAULT_SENSITIVITY_X * config.gamepad_sensitivity_x,
        DEFAULT_SENSITIVITY_Y * config.gamepad_sensitivity_y,
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
pub(crate) fn emit_ui_confirm_back_messages(
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
