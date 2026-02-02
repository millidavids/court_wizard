use bevy::prelude::*;

use super::constants::*;
use super::events::*;
use super::resources::*;
use crate::game::units::wizard::components::PrimeSpellMessage;

/// Detects rune key presses (Q, W, E, R) and sends RunePressed messages.
/// Only runs during InGameState::Running.
pub fn detect_rune_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut rune_pressed: MessageWriter<RunePressed>,
) {
    for rune in [Rune::Q, Rune::W, Rune::E, Rune::R] {
        if keyboard.just_pressed(rune.keycode()) {
            rune_pressed.write(RunePressed { rune });
        }
    }
}

/// Handles RunePressed messages by adding to the current sequence.
pub fn handle_rune_pressed(
    mut messages: MessageReader<RunePressed>,
    mut sequence: ResMut<RuneSequence>,
) {
    for message in messages.read() {
        // Don't exceed max length
        if sequence.len() < MAX_RUNE_SEQUENCE_LENGTH {
            sequence.push(message.rune);
        }
    }
}

/// Detects spacebar press for activating rune sequences.
pub fn detect_rune_activation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut activate: MessageWriter<ActivateRuneSequence>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        activate.write(ActivateRuneSequence);
    }
}

/// Handles rune sequence activation by mapping to spell and sending PrimeSpellMessage.
pub fn handle_rune_activation(
    mut messages: MessageReader<ActivateRuneSequence>,
    mut sequence: ResMut<RuneSequence>,
    mut prime_spell: MessageWriter<PrimeSpellMessage>,
) {
    for _ in messages.read() {
        if let Some(spell) = sequence_to_spell(&sequence.runes) {
            // Valid sequence - prime the spell
            prime_spell.write(PrimeSpellMessage {
                spell: spell.primed_config(),
            });
        }
        // Always clear sequence after activation attempt (valid or invalid)
        sequence.clear();
    }
}

/// Updates the rune sequence idle timer and clears on timeout.
pub fn update_rune_timeout(time: Res<Time>, mut sequence: ResMut<RuneSequence>) {
    sequence.tick(time.delta_secs());

    if sequence.has_timed_out(SEQUENCE_TIMEOUT_DURATION) {
        sequence.clear();
    }
}
