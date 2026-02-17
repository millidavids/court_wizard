use bevy::prelude::*;

/// Message sent when an enemy dies, requesting a potential ingredient drop spawn.
#[derive(Message)]
pub(crate) struct SpawnIngredientDropMessage {
    /// World position where the enemy died (XZ used, Y ignored).
    pub position: Vec3,
}
