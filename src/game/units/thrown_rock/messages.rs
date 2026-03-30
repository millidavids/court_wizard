use bevy::prelude::*;

/// Message sent when a brute or ogre throws a rock.
#[derive(Message)]
pub struct RockThrownMessage {
    /// Position where the thrower is standing.
    pub origin: Vec3,
    /// Target landing position.
    pub target: Vec3,
}
