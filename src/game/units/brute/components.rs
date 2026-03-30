use bevy::prelude::*;

/// Marker component for brute units.
#[derive(Component)]
pub struct Brute;

/// Cooldown timer for the brute's rock throw ability.
#[derive(Component)]
pub struct RockThrowCooldown {
    pub time_remaining: f32,
}

impl RockThrowCooldown {
    pub fn new(cooldown: f32) -> Self {
        Self {
            time_remaining: cooldown,
        }
    }

    pub fn tick(&mut self, delta: f32) {
        self.time_remaining -= delta;
    }

    pub fn is_ready(&self) -> bool {
        self.time_remaining <= 0.0
    }

    pub fn reset(&mut self, cooldown: f32) {
        self.time_remaining = cooldown;
    }
}
