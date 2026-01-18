use bevy::prelude::*;

// Entity Colors
pub const DEFENDER_COLOR: Color = Color::srgb(0.9, 0.9, 0.2); // Yellow

// Entity Sizes
pub const UNIT_RADIUS: f32 = 15.0; // Circle radius for units

// Gameplay Constants
pub const UNIT_SPEED: f32 = 100.0; // Pixels per second
pub const SPAWN_INTERVAL: f32 = 2.0; // Seconds between spawns
pub const COMBAT_RANGE: f32 = 30.0; // Distance for combat interaction
