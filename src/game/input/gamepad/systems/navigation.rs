use bevy::prelude::*;

use super::super::constants::{
    DEFAULT_SENSITIVITY_X, DEFAULT_SENSITIVITY_Y, RADIAL_SLOT_COUNT, RADIAL_WEDGE_DEGREES,
};
use super::super::messages::{GamepadConfirmPressed, MenuBackPressed};
use super::super::resources::{
    ActiveInputDevice, GamepadAimSettings, RadialHoveredSlot, VirtualCursorPosition,
};
use crate::game::input::action_state::{GamepadAction, GamepadActionState};
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
/// `PrimaryCast`/`SecondaryCast` are digital actions (the active producer applies
/// the trigger threshold), so edge detection comes from `GamepadActionState`'s
/// `ButtonInput` — no manual `Local` trigger tracking. The `commit_armed` `Local`
/// still suppresses `MouseLeftPressed/Held/Released` for the lifetime of a radial
/// commit press.
#[allow(clippy::too_many_arguments)]
pub(crate) fn translate_triggers_to_mouse_messages(
    state: Res<GamepadActionState>,
    hovered_slot: Res<RadialHoveredSlot>,
    virtual_cursor: Res<VirtualCursorPosition>,
    mut commit_armed: Local<bool>,
    mut action_bar: MessageWriter<ActionBarKeyPressed>,
    mut left_pressed: MessageWriter<MouseLeftPressed>,
    mut left_held: MessageWriter<MouseLeftHeld>,
    mut left_released: MessageWriter<MouseLeftReleased>,
    mut right_pressed: MessageWriter<MouseRightPressed>,
    mut right_held: MessageWriter<MouseRightHeld>,
    mut right_released: MessageWriter<MouseRightReleased>,
) {
    let cursor_position = Some(virtual_cursor.screen_pos);

    // Right trigger → primary cast → left mouse (or radial commit).
    if state.just_pressed(GamepadAction::PrimaryCast) {
        if let Some(slot) = hovered_slot.0 {
            action_bar.write(ActionBarKeyPressed { slot });
            *commit_armed = true;
        } else {
            left_pressed.write(MouseLeftPressed { cursor_position });
            left_held.write(MouseLeftHeld { cursor_position });
        }
    } else if state.pressed(GamepadAction::PrimaryCast) {
        if !*commit_armed {
            left_held.write(MouseLeftHeld { cursor_position });
        }
    } else if state.just_released(GamepadAction::PrimaryCast) {
        if *commit_armed {
            *commit_armed = false;
        } else {
            left_released.write(MouseLeftReleased);
        }
    }

    // Left trigger → secondary cast → right mouse.
    if state.just_pressed(GamepadAction::SecondaryCast) {
        right_pressed.write(MouseRightPressed { cursor_position });
        right_held.write(MouseRightHeld { cursor_position });
    } else if state.pressed(GamepadAction::SecondaryCast) {
        right_held.write(MouseRightHeld { cursor_position });
    } else if state.just_released(GamepadAction::SecondaryCast) {
        right_released.write(MouseRightReleased);
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
    state: Res<GamepadActionState>,
    aim: Res<GamepadAimSettings>,
    mut hovered: ResMut<RadialHoveredSlot>,
) {
    if !active.is_gamepad() {
        if hovered.0.is_some() {
            hovered.0 = None;
        }
        return;
    }
    let new_slot = right_stick_to_slot(state.right_stick, aim.deadzone);
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
    state: Res<GamepadActionState>,
    mut confirm: MessageWriter<GamepadConfirmPressed>,
    mut back: MessageWriter<MenuBackPressed>,
) {
    if state.just_pressed(GamepadAction::UIConfirm) {
        confirm.write(GamepadConfirmPressed);
    }
    if state.just_pressed(GamepadAction::UIBack) {
        back.write(MenuBackPressed);
    }
}
