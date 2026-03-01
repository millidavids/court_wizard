use bevy::prelude::*;

/// Cooldown timer for wizard mind control casting.
#[derive(Component)]
pub struct MindControlCooldown {
    pub remaining: f32,
}
