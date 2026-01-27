use bevy::prelude::*;

/// Marker component for all game entities (cleanup on exit from InGame state).
#[derive(Component)]
pub struct OnGameplayScreen;

/// Marker component for entities that should always face the camera (billboard effect).
///
/// Entities with this component will have their rotation updated each frame to face the camera,
/// rotating around the Y axis to remain perpendicular to the camera's forward direction on the XZ plane.
#[derive(Component)]
pub struct Billboard;

/// Velocity component for moving units.
///
/// Represents the unit's movement speed on the XZ plane (units per second).
/// Units don't move vertically - they stay at their spawn height.
/// Z velocity controls depth movement (toward/away from camera).
#[derive(Component, Default)]
pub struct Velocity {
    pub x: f32,
    pub z: f32,
}

/// Acceleration component for units using boids flocking.
///
/// Represents forces applied to the unit on the XZ plane. Acceleration is reset each frame.
/// Units don't accelerate vertically - they stay at their spawn height.
#[derive(Component)]
pub struct Acceleration {
    pub x: f32,
    pub z: f32,
}

impl Acceleration {
    pub const fn new() -> Self {
        Self { x: 0.0, z: 0.0 }
    }

    pub fn reset(&mut self) {
        self.x = 0.0;
        self.z = 0.0;
    }

    pub fn add_force(&mut self, force: Vec3) {
        self.x += force.x;
        self.z += force.z;
        // Ignore Y component - units only move on XZ plane
    }
}
