use bevy::prelude::*;

use super::brews::Recipe;

/// Message sent when the player selects ingredients and starts brewing.
#[derive(Message, Debug, Clone)]
pub struct StartBrewMessage {
    pub recipe: Recipe,
}

/// Message sent when a brew finishes and is ready to apply its buff.
#[derive(Message, Debug, Clone)]
pub struct BrewCompleteMessage {
    pub recipe: Recipe,
}

/// Message sent when the player cancels an in-progress brew.
#[derive(Message, Debug, Clone, Copy)]
pub struct CancelBrewMessage;
