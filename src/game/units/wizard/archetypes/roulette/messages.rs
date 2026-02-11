use bevy::prelude::*;

/// Message sent when the player triggers a roulette spin (spacebar).
#[derive(Message, Debug, Clone, Copy)]
pub struct RouletteSpinMessage;
