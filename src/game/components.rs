use bevy::prelude::*;

/// Marker component for all game entities (cleanup on exit from InGame state).
#[derive(Component)]
pub struct OnGameplayScreen;

/// Velocity component for moving units.
///
/// Represents the unit's movement speed in 3D space (units per second).
/// Z velocity controls depth movement (toward/away from camera).
#[derive(Component, Default)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Acceleration component for units using boids flocking.
///
/// Represents forces applied to the unit. Acceleration is reset each frame.
#[derive(Component)]
pub struct Acceleration {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Acceleration {
    pub const fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.z = 0.0;
    }

    pub fn add_force(&mut self, force: Vec3) {
        self.x += force.x;
        self.y += force.y;
        self.z += force.z;
    }
}
