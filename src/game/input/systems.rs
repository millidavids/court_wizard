//! Input detection systems.
//!
//! These systems query input state once per frame and send events
//! that other systems can consume.

use bevy::prelude::*;

use super::{
    components::{
        MouseButtonState, MouseLeftHeldThisFrame, MouseRightHeldThisFrame,
        SpellInputBlockedThisFrame,
    },
    events::*,
};

/// Detects mouse button input and sends events.
///
/// Runs once per frame to query mouse state and fire appropriate events.
#[allow(clippy::too_many_arguments)]
pub fn detect_mouse_input(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut left_pressed: MessageWriter<MouseLeftPressed>,
    mut left_held: MessageWriter<MouseLeftHeld>,
    mut left_released: MessageWriter<MouseLeftReleased>,
    mut right_pressed: MessageWriter<MouseRightPressed>,
    mut right_held: MessageWriter<MouseRightHeld>,
    mut right_released: MessageWriter<MouseRightReleased>,
) {
    // Get cursor position from primary window
    let cursor_position = windows
        .single()
        .ok()
        .and_then(|window| window.cursor_position());

    // Check left mouse button state
    if mouse.just_pressed(MouseButton::Left) {
        left_pressed.write(MouseLeftPressed { cursor_position });
    }

    if mouse.pressed(MouseButton::Left) {
        left_held.write(MouseLeftHeld { cursor_position });
    }

    if mouse.just_released(MouseButton::Left) {
        left_released.write(MouseLeftReleased);
    }

    // Only clear consumed flag when button is completely idle (not pressed, not released this frame)
    if !mouse.pressed(MouseButton::Left) && !mouse.just_released(MouseButton::Left) {
        mouse_state.left_consumed = false;
    }

    // Check right mouse button state
    if mouse.just_pressed(MouseButton::Right) {
        right_pressed.write(MouseRightPressed { cursor_position });
    }

    if mouse.pressed(MouseButton::Right) {
        right_held.write(MouseRightHeld { cursor_position });
    }

    if mouse.just_released(MouseButton::Right) {
        right_released.write(MouseRightReleased);
    }
}

/// Detects keyboard input and sends events.
///
/// Runs once per frame to query keyboard state and fire appropriate events.
pub fn detect_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut spacebar_pressed: MessageWriter<SpacebarPressed>,
    mut spacebar_held: MessageWriter<SpacebarHeld>,
    mut spacebar_released: MessageWriter<SpacebarReleased>,
) {
    // Check spacebar state
    if keyboard.just_pressed(KeyCode::Space) {
        spacebar_pressed.write(SpacebarPressed);
    }

    if keyboard.pressed(KeyCode::Space) {
        spacebar_held.write(SpacebarHeld);
    }

    if keyboard.just_released(KeyCode::Space) {
        spacebar_released.write(SpacebarReleased);
    }
}

/// Updates frame-based input state resources for run conditions.
///
/// This system consumes input messages and stores their state in resources
/// that can be safely queried by run_if conditions. Must run BEFORE spell systems.
pub fn update_input_state_for_run_conditions(
    mut block_spell_input: MessageReader<BlockSpellInput>,
    mut mouse_left_held: MessageReader<MouseLeftHeld>,
    mut mouse_right_held: MessageReader<MouseRightHeld>,
    mut spell_blocked: ResMut<SpellInputBlockedThisFrame>,
    mut mouse_left_held_state: ResMut<MouseLeftHeldThisFrame>,
    mut mouse_right_held_state: ResMut<MouseRightHeldThisFrame>,
) {
    spell_blocked.blocked = block_spell_input.read().next().is_some();
    mouse_left_held_state.held = mouse_left_held.read().next().is_some();
    mouse_right_held_state.held = mouse_right_held.read().next().is_some();
}
