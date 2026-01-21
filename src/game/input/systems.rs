//! Input detection systems.
//!
//! These systems query input state once per frame and send events
//! that other systems can consume.

use bevy::prelude::*;

use super::events::*;

/// Detects mouse button input and sends events.
///
/// Runs once per frame to query mouse state and fire appropriate events.
pub fn detect_mouse_input(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut left_pressed: MessageWriter<MouseLeftPressed>,
    mut left_held: MessageWriter<MouseLeftHeld>,
    mut left_released: MessageWriter<MouseLeftReleased>,
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
