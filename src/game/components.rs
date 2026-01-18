use bevy::prelude::*;

/// Marker component for all game entities (cleanup on exit from InGame state).
#[derive(Component)]
pub struct OnGameplayScreen;

/// Velocity component for moving units.
///
/// Represents the unit's movement speed in 3D space (units per second).
/// Z velocity controls depth movement (toward/away from camera).
#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
