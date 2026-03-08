use bevy::prelude::*;

/// Broadcast when the active weather changes.
#[derive(Message, Debug, Clone, Copy)]
pub struct WeatherChangedMessage;
