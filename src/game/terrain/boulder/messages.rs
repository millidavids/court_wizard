use bevy::prelude::*;

/// Message sent when a brute or ogre throws a boulder.
#[derive(Message)]
pub struct BoulderThrownMessage {
    /// Position where the thrower is standing.
    pub origin: Vec3,
    /// Target landing position.
    pub target: Vec3,
}
