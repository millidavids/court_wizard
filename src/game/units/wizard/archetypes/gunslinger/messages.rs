use bevy::prelude::*;

use super::components::GunType;

/// Message sent when the player selects a gun (keys 1-5 or slot click).
#[derive(Message, Debug, Clone, Copy)]
pub struct SelectGunMessage {
    pub gun: GunType,
}

/// Message sent when the player requests a manual reload (R key).
#[derive(Message, Debug, Clone, Copy)]
pub struct ReloadMessage;
