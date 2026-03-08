//! Input detection systems.
//!
//! These systems query input state once per frame and send messages
//! that other systems can consume.

use bevy::prelude::*;

use super::{
    components::{
        MouseButtonState, MouseLeftHeldThisFrame, MouseRightHeldThisFrame,
        SpellInputBlockedThisFrame,
    },
    messages::*,
};
use crate::game::crt_effect::CorrectedCursorPosition;

/// Clears all mouse input state to prevent stale events from carrying across state transitions.
///
/// Runs on entering `InGameState::Running` to ensure menu clicks don't trigger
/// spell casts when the game begins.
pub fn clear_mouse_input_state(
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_held_state: ResMut<MouseLeftHeldThisFrame>,
    mut mouse_right_held_state: ResMut<MouseRightHeldThisFrame>,
) {
    mouse.clear();
    mouse_state.left_consumed = false;
    mouse_left_held_state.held = false;
    mouse_right_held_state.held = false;
}

/// Detects mouse button input and sends messages.
///
/// Runs once per frame to query mouse state and fire appropriate messages.
#[allow(clippy::too_many_arguments)]
pub fn detect_mouse_input(
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut left_pressed: MessageWriter<MouseLeftPressed>,
    mut left_held: MessageWriter<MouseLeftHeld>,
    mut left_released: MessageWriter<MouseLeftReleased>,
    mut right_pressed: MessageWriter<MouseRightPressed>,
    mut right_held: MessageWriter<MouseRightHeld>,
    mut right_released: MessageWriter<MouseRightReleased>,
) {
    // Use barrel-distortion-corrected cursor position
    let cursor_position = corrected_cursor.0;

    // Check left mouse button state
    // If button is pressed but we're not getting a just_pressed event, it's stuck from losing focus
    if mouse.pressed(MouseButton::Left) && !mouse.just_pressed(MouseButton::Left) {
        // Check if we're about to send held events - if so, button is legitimately held
        // If not, it's stuck and we should clear it
        if cursor_position.is_none() {
            // No cursor means window doesn't have focus, clear the stuck state
            mouse.clear();
            mouse_state.left_consumed = false;
            return;
        }
    }

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

/// Detects keyboard input and sends messages.
///
/// Runs once per frame to query keyboard state and fire appropriate messages.
/// This system handles universal inputs (spacebar, action bar keys).
/// Archetype-specific inputs are handled by `detect_rune_input` and `detect_roulette_input`.
pub fn detect_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut spacebar_pressed: MessageWriter<SpacebarPressed>,
    mut spacebar_held: MessageWriter<SpacebarHeld>,
    mut spacebar_released: MessageWriter<SpacebarReleased>,
    mut action_bar_pressed: MessageWriter<ActionBarKeyPressed>,
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

    // Check number keys 1-5 (for slots 0-4 in action bar)
    const NUMBER_KEYS: [(KeyCode, u8); 5] = [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
    ];

    for (key_code, slot) in NUMBER_KEYS {
        if keyboard.just_pressed(key_code) {
            action_bar_pressed.write(ActionBarKeyPressed { slot });
        }
    }
}

/// Detects rune key input (Q/W/E/R and spacebar to activate) for the RuneCaster archetype.
///
/// This system is gated to only run when the active wizard type is RuneCaster.
pub fn detect_rune_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut rune_pressed: MessageWriter<
        crate::game::units::wizard::archetypes::runes::messages::RunePressed,
    >,
    mut rune_activate: MessageWriter<
        crate::game::units::wizard::archetypes::runes::messages::ActivateRuneSequence,
    >,
) {
    // Spacebar activates rune sequence
    if keyboard.just_pressed(KeyCode::Space) {
        rune_activate
            .write(crate::game::units::wizard::archetypes::runes::messages::ActivateRuneSequence);
    }

    // Check rune keys Q, W, E, R
    const RUNE_KEYS: [(
        KeyCode,
        crate::game::units::wizard::archetypes::runes::resources::Rune,
    ); 4] = [
        (
            KeyCode::KeyQ,
            crate::game::units::wizard::archetypes::runes::resources::Rune::Q,
        ),
        (
            KeyCode::KeyW,
            crate::game::units::wizard::archetypes::runes::resources::Rune::W,
        ),
        (
            KeyCode::KeyE,
            crate::game::units::wizard::archetypes::runes::resources::Rune::E,
        ),
        (
            KeyCode::KeyR,
            crate::game::units::wizard::archetypes::runes::resources::Rune::R,
        ),
    ];

    for (key_code, rune) in RUNE_KEYS {
        if keyboard.just_pressed(key_code) {
            rune_pressed.write(
                crate::game::units::wizard::archetypes::runes::messages::RunePressed { rune },
            );
        }
    }
}

/// Detects spacebar press to trigger a roulette spin for the Randomancer archetype.
///
/// This system is gated to only run when the active wizard type is Randomancer.
pub fn detect_roulette_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut spin_message: MessageWriter<
        crate::game::units::wizard::archetypes::roulette::messages::RouletteSpinMessage,
    >,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        spin_message
            .write(crate::game::units::wizard::archetypes::roulette::messages::RouletteSpinMessage);
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
