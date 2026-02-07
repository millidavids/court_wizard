use bevy::prelude::*;

/// Message that triggers saving the current configuration to localStorage.
///
/// Send this message when you want to manually persist the current
/// config state immediately, bypassing the debounce timer.
#[derive(Message)]
pub(crate) struct SaveConfigMessage;

/// Message that triggers debounced config save.
///
/// Send this message whenever any configuration changes that should be
/// persisted to localStorage. The ConfigPlugin will debounce these messages
/// and save after 0.5s of inactivity.
#[derive(Message)]
pub struct ConfigChanged;
