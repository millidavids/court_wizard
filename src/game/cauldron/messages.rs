use bevy::prelude::*;

use super::brews::Brew;

/// Message sent when the player selects a brew to start.
#[derive(Message, Debug, Clone, Copy)]
pub struct StartBrewMessage {
    pub brew: Brew,
}

/// Message sent when a brew finishes and is ready to apply its buff.
#[derive(Message, Debug, Clone, Copy)]
pub struct BrewCompleteMessage {
    pub brew: Brew,
}

/// Message sent when the player cancels an in-progress brew.
#[derive(Message, Debug, Clone, Copy)]
pub struct CancelBrewMessage;
