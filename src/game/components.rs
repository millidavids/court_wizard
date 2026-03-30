use bevy::prelude::*;

/// Marker component for all game entities (cleanup on exit from InGame state).
#[derive(Component, Clone)]
pub struct OnGameplayScreen;

/// Health component for destructible obstacles (walls, rocks).
#[derive(Component)]
pub struct ObstacleHealth {
    pub current: f32,
    pub max: f32,
}

impl ObstacleHealth {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn fraction(&self) -> f32 {
        if self.max <= 0.0 {
            return 0.0;
        }
        self.current / self.max
    }
}

/// Marker component for spells that require concentration to maintain.
///
/// When a spell with this component is active, the concentration UI will be shown
/// and the wizard cannot cast other spells without ending concentration.
#[derive(Component)]
pub struct ConcentrationSpell {
    /// Name of the spell being concentrated on.
    pub spell_name: &'static str,
}

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
    /// Maximum allowed speed magnitude. Set by the movement system each frame.
    /// When > 0, velocity is clamped to this magnitude after acceleration integration.
    pub max_speed: f32,
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
