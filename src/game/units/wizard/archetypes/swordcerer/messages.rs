use bevy::prelude::*;

/// Sent when the swordcerer avatar dies or the player wants to retreat.
#[derive(Message, Debug, Clone, Copy)]
pub struct RetreatMessage;
