use bevy::prelude::*;

use super::resources::Rune;

/// Message sent when a rune button is pressed.
#[derive(Message, Debug, Clone, Copy)]
pub struct RunePressed {
    pub rune: Rune,
}

/// Message sent when spacebar is pressed to activate the rune sequence.
#[derive(Message, Debug, Clone, Copy)]
pub struct ActivateRuneSequence;
