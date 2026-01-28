// Movement
pub const ARCHER_MOVEMENT_SPEED: f32 = 100.0; // Significantly slower than infantry (200)

// Attack Range
pub const ARCHER_MIN_RANGE: f32 = 150.0; // Optimal minimum distance
pub const ARCHER_MAX_RANGE: f32 = 700.0; // Maximum attack range

// Combat
pub const ARCHER_ATTACK_DAMAGE: f32 = 30.0; // Arrow damage (high damage but slow fire rate)
pub const ARCHER_MELEE_DAMAGE: f32 = 5.0; // Much less than infantry (10)
pub const ARCHER_ATTACK_DELAY_AFTER_MOVEMENT: f32 = 1.0; // Seconds to wait after stopping before attacking
pub const ARCHER_ATTACK_COOLDOWN_MULTIPLIER: f32 = 2.0; // Archers attack half as often as melee units

// Arrow Projectile
pub const ARROW_GRAVITY: f32 = 600.0; // Downward acceleration (lower = more arc)
pub const ARROW_LAUNCH_ANGLE_DEGREES: f32 = 30.0; // Launch angle above horizontal
pub const ARROW_WIDTH: f32 = 4.0; // Visual radius (circle)
pub const ARROW_POWER_VARIATION: f32 = 0.05; // ±5% power variation
pub const ARROW_ANGLE_VARIATION_DEGREES: f32 = 1.0; // ±1 degree angle variation

// Spawn counts (for initial testing)
pub const INITIAL_ARCHER_DEFENDER_COUNT: u32 = 20;
pub const INITIAL_ARCHER_ATTACKER_COUNT: u32 = 20;
