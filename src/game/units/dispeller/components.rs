use bevy::prelude::*;

#[derive(Component)]
pub struct Dispeller;

#[derive(Component)]
pub struct DispellerDispelCooldown {
    pub remaining: f32,
}
