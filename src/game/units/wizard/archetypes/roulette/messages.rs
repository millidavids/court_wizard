use bevy::prelude::*;

use crate::game::units::wizard::components::Spell;

/// Message sent when the player triggers a roulette spin (spacebar).
#[derive(Message, Debug, Clone, Copy)]
pub struct RouletteSpinMessage;

/// Message sent when the roulette wheel lands on a spell.
#[derive(Message, Debug, Clone, Copy)]
pub struct RouletteSelectedMessage {
    pub spell: Spell,
}
