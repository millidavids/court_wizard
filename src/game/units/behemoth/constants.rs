use bevy::prelude::*;

// Visual appearance
pub const BEHEMOTH_COLOR: Color = Color::srgb(0.4, 0.2, 0.5); // Dark purple
pub const BEHEMOTH_ELLIPSE_WIDTH: f32 = 20.0; // Ellipse width (X axis)
pub const BEHEMOTH_ELLIPSE_DEPTH: f32 = 30.0; // Ellipse depth (Z axis) - longer oval
pub const BEHEMOTH_RADIUS: f32 = BEHEMOTH_ELLIPSE_WIDTH / 2.0; // Horizontal radius (10.0) for collision detection
pub const BEHEMOTH_HITBOX_HEIGHT: f32 = 35.0; // Vertical height (taller than regular attackers)

// Movement
pub const BEHEMOTH_MOVEMENT_SPEED: f32 = 75.0; // Slower than infantry (100.0) for tank-like feel

// Combat
pub const BEHEMOTH_HEALTH: f32 = 500.0; // 4x normal unit health (50.0 * 4)
pub const BEHEMOTH_AOE_DAMAGE: f32 = 200.0; // AOE splash damage
pub const BEHEMOTH_AOE_RADIUS: f32 = 30.0; // AOE effect radius

// Spawn interval
pub const BEHEMOTH_SPAWN_LEVEL_INTERVAL: u32 = 1; // Spawn every level (for testing)
