// Movement — 98% of infantry base speed.
pub const ARCHER_MOVEMENT_SPEED: f32 = crate::game::constants::UNIT_MOVEMENT_SPEED * 0.98;

// Attack Range
pub const ARCHER_MIN_RANGE: f32 = 150.0; // Optimal minimum distance
pub const ARCHER_MAX_RANGE: f32 = 700.0; // Maximum attack range
pub const ARCHER_SEEK_RANGE: f32 = 500.0; // Archers advance until enemies are this close, then stop

// Combat
pub const ARCHER_ATTACK_DAMAGE: f32 = 30.0; // Arrow damage (high damage but slow fire rate)
pub const ARCHER_MELEE_DAMAGE: f32 = 5.0; // Much less than infantry (10)
pub const ARCHER_ATTACK_DELAY_AFTER_MOVEMENT: f32 = 1.0; // Seconds to wait after stopping before attacking
pub const ARCHER_ATTACK_COOLDOWN_MULTIPLIER: f32 = 2.0; // Archers attack half as often as melee units

// Arrow Projectile
pub const ARROW_GRAVITY: f32 = 600.0; // Downward acceleration (lower = more arc)
pub const ARROW_LAUNCH_ANGLE_DEGREES: f32 = 30.0; // Launch angle above horizontal
pub const ARROW_WIDTH: f32 = 3.0; // Visual width (rectangle)
pub const ARROW_LENGTH: f32 = 16.0; // Visual length (rectangle)
pub const ARROW_POWER_VARIATION: f32 = 0.05; // ±5% power variation
pub const ARROW_ANGLE_VARIATION_DEGREES: f32 = 1.0; // ±1 degree angle variation

// Wall interaction
/// How far from the approach path a wall center can be to prevent shooting stance.
pub const WALL_APPROACH_PATH_BUFFER: f32 = 50.0;

// Spawn counts (for initial testing)
pub const INITIAL_ARCHER_DEFENDER_COUNT: u32 = 20;

// --- Visual styles ---

use bevy::prelude::*;

use crate::game::constants::UNIT_SCALE;

// Arrow
pub const ARROW_COLOR: Color = Color::srgb(0.3, 0.2, 0.1); // Dark brown
pub const ARCHER_RADIUS: f32 = 8.0 * UNIT_SCALE; // Same as infantry

// Sprite animation (re-export shared defaults for backward compatibility)
pub use crate::game::units::constants::DEFAULT_SPRITE_HEIGHT as ARCHER_SPRITE_HEIGHT;
pub use crate::game::units::constants::DEFAULT_SPRITE_WIDTH as ARCHER_SPRITE_WIDTH;

// Sprite tint colors
pub use crate::game::units::infantry::constants::DEFENDER_SPRITE_TINT;
/// Lighter attacker tint for archers (infantry uses darker 0.55/0.45/0.45).
pub const ATTACKER_SPRITE_TINT: Color = Color::srgb(0.75, 0.65, 0.65);
