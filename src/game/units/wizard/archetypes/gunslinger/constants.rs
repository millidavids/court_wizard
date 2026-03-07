// ===== Machine Gun =====
pub const MACHINE_GUN_MAX_AMMO: u32 = 60;
pub const MACHINE_GUN_RELOAD_TIME: f32 = 2.0;
pub const MACHINE_GUN_DAMAGE: f32 = 25.0;
pub const MACHINE_GUN_FIRE_INTERVAL: f32 = 0.08;
pub const MACHINE_GUN_BULLET_SPEED: f32 = 20000.0;
pub const MACHINE_GUN_SPREAD: f32 = 0.04; // radians

// ===== Magnum =====
pub const MAGNUM_MAX_AMMO: u32 = 6;
pub const MAGNUM_RELOAD_TIME: f32 = 1.5;
pub const MAGNUM_DAMAGE: f32 = 100.0;
pub const MAGNUM_FIRE_INTERVAL: f32 = 0.4;
pub const MAGNUM_BULLET_SPEED: f32 = 25000.0;

// ===== Rocket Launcher =====
pub const ROCKET_MAX_AMMO: u32 = 3;
pub const ROCKET_RELOAD_TIME: f32 = 3.0;
pub const ROCKET_DAMAGE: f32 = 100.0;
pub const ROCKET_EXPLOSION_RADIUS: f32 = 120.0;
pub const ROCKET_SPEED: f32 = 1500.0;
pub const ROCKET_FIRE_INTERVAL: f32 = 0.6;

// ===== Shotgun =====
pub const SHOTGUN_MAX_AMMO: u32 = 8;
pub const SHOTGUN_RELOAD_TIME: f32 = 2.5;
pub const SHOTGUN_PELLET_DAMAGE: f32 = 10.0;
pub const SHOTGUN_PELLET_COUNT: u32 = 30;
pub const SHOTGUN_SPREAD: f32 = 0.075; // radians (cone half-angle)
pub const SHOTGUN_FIRE_INTERVAL: f32 = 0.5;
pub const SHOTGUN_BULLET_SPEED: f32 = 15000.0;

// ===== Flamethrower =====
pub const FLAMETHROWER_MAX_AMMO: u32 = 100;
pub const FLAMETHROWER_RELOAD_TIME: f32 = 2.5;
pub const FLAMETHROWER_DAMAGE: f32 = 1.5;
pub const FLAMETHROWER_FIRE_INTERVAL: f32 = 0.05;
pub const FLAMETHROWER_SPEED: f32 = 2000.0;
pub const FLAMETHROWER_SPREAD: f32 = 0.08; // radians
pub const FLAMETHROWER_GRAVITY: f32 = 600.0;
pub const FLAMETHROWER_PARTICLE_LIFETIME: f32 = 5.0;
pub const BURNING_GROUND_RADIUS: f32 = 40.0;
pub const BURNING_GROUND_DAMAGE: f32 = 1.0;
pub const BURNING_GROUND_DURATION: f32 = 3.0;

// ===== Hitscan =====
/// Radius of the invisible hitscan cylinder for collision detection.
pub const HITSCAN_CYLINDER_RADIUS: f32 = 25.0;

// ===== Bullet Visuals =====
pub const BULLET_RADIUS: f32 = 2.0;
pub const BULLET_LENGTH: f32 = 15.0;
pub const BULLET_HIT_FLASH_DURATION: f32 = 0.08;
pub const ROCKET_RADIUS: f32 = 8.0;
pub const FLAME_PARTICLE_START_SIZE: f32 = 10.0;
pub const FLAME_PARTICLE_SIZE: f32 = 15.0;

// ===== Muzzle Flash =====
pub const MUZZLE_FLASH_DURATION: f32 = 0.06;
pub const MUZZLE_FLASH_SIZE: f32 = 20.0;
