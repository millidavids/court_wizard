use bevy::prelude::*;

// Visual appearance
/// Radius of the boulder obstacle on the battlefield (world units).
pub const ROCK_RADIUS: f32 = 35.0;
/// Height of the boulder obstacle for collision checks.
pub const ROCK_HEIGHT: f32 = 50.0;
/// Visual scale of the boulder circle mesh.
pub const ROCK_VISUAL_RADIUS: f32 = 35.0;
/// Color of a full-health boulder.
pub const ROCK_BASE_COLOR: Color = Color::srgba(0.55, 0.53, 0.50, 1.0);
/// Color of a boulder at zero HP.
pub const ROCK_DAMAGED_COLOR: Color = Color::srgba(0.35, 0.30, 0.25, 1.0);

// Health
/// Hit points for a thrown boulder.
pub const ROCK_HEALTH: f32 = 300.0;

// Combat
/// Damage dealt by units per melee hit against a blocking boulder.
pub const ROCK_DAMAGE_PER_HIT: f32 = 25.0;
/// Distance from boulder surface within which a unit can attack it.
pub const ROCK_ATTACK_RANGE: f32 = 30.0;

// Boulder throw
/// Cooldown between boulder throws (seconds). Used by both brute and ogre.
pub const ROCK_THROW_COOLDOWN: f32 = 15.0;
/// Maximum range for throwing a boulder.
pub const ROCK_THROW_RANGE: f32 = 250.0;
/// Duration of the boulder projectile arc (seconds).
pub const ROCK_PROJECTILE_DURATION: f32 = 0.8;
/// Peak height of the parabolic arc (world units above ground).
pub const ROCK_PROJECTILE_ARC_HEIGHT: f32 = 120.0;

// Destruction animation
/// Duration of the sinking animation when a boulder is destroyed (seconds).
pub const ROCK_SINK_DURATION: f32 = 2.0;

// Overlap resolution
/// Minimum separation between two boulder centers (prevents overlapping).
pub const ROCK_MIN_SEPARATION: f32 = ROCK_RADIUS * 2.0 + 5.0;

// Shadow
/// Y position for boulder shadows (just above ground to avoid z-fighting).
pub const ROCK_SHADOW_Y: f32 = 1.5;
/// Shadow scale relative to boulder visual radius.
pub const ROCK_SHADOW_SCALE: f32 = 1.2;
