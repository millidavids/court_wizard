use bevy::prelude::*;

use super::constants::*;
use super::messages::*;
use super::resources::*;
use crate::game::units::wizard::messages::PrimeSpellMessage;

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

/// Handles rune sequence activation by mapping to spell and sending PrimeSpellMessage.
pub fn handle_rune_activation(
    mut messages: MessageReader<ActivateRuneSequence>,
    mut sequence: ResMut<RuneSequence>,
    mut last_activated: ResMut<super::resources::LastActivatedSpell>,
    mut prime_spell: MessageWriter<PrimeSpellMessage>,
    mut spell_activated: MessageWriter<RuneSpellActivated>,
) {
    for _ in messages.read() {
        if let Some(spell) = sequence_to_spell(&sequence.runes) {
            // Store in resource for UI display FIRST, before clearing sequence
            last_activated.activate(spell);

            // Valid sequence - prime the spell with 25% empowerment
            prime_spell.write(PrimeSpellMessage {
                spell: spell.primed_config().with_empowerment(1.25),
            });
            // Notify UI that a spell was activated
            spell_activated.write(RuneSpellActivated { spell });
        }
        // Always clear sequence after activation attempt (valid or invalid)
        // By this point, last_activated is already set, so UI won't overwrite it
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
