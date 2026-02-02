use bevy::prelude::*;

use super::resources::Rune;
use crate::game::units::wizard::components::Spell;

/// Message sent when a rune button is pressed.
#[derive(Message, Debug, Clone, Copy)]
pub struct RunePressed {
    pub rune: Rune,
}

/// Message sent when spacebar is pressed to activate the rune sequence.
#[derive(Message, Debug, Clone, Copy)]
pub struct ActivateRuneSequence;

/// Message sent when a valid rune sequence is activated with the resulting spell.
#[derive(Message, Debug, Clone, Copy)]
pub struct RuneSpellActivated {
    pub spell: Spell,
}
