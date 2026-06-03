use bevy::prelude::*;

use crate::game::constants::UNIT_SCALE;

/// Movement speed for the swordcerer avatar (8x normal infantry speed).
pub(super) const AVATAR_MOVEMENT_SPEED: f32 = 920.0;

/// Health of the swordcerer avatar on the field.
pub(super) const AVATAR_HEALTH: f32 = 150.0;

/// Sprite tint for the swordcerer avatar (neutral white — swordcerer has its own sprite sheet).
pub(super) const AVATAR_SPRITE_TINT: Color = Color::WHITE;

pub(super) use crate::game::units::constants::DEFAULT_SPRITE_HEIGHT as AVATAR_SPRITE_HEIGHT;
/// Sprite dimensions for the swordcerer avatar.
pub(super) use crate::game::units::constants::DEFAULT_SPRITE_WIDTH as AVATAR_SPRITE_WIDTH;

/// Magic missile damage from the swordcerer avatar.
pub(super) const MISSILE_DAMAGE: f32 = 20.0;

/// Mana cost per magic missile fired from the field.
pub(super) const MISSILE_MANA_COST: f32 = 8.0;

/// Magic missile cooldown in seconds.
pub(super) const MISSILE_COOLDOWN: f32 = 0.25;

/// Magic missile speed.
pub(super) const MISSILE_SPEED: f32 = 800.0;

/// Base homing force for avatar missiles. ~10× the default magic-missile
/// homing — anything weaker lets the missile orbit close-range targets at
/// the avatar's higher fire speed instead of impacting.
pub(super) const MISSILE_HOMING_STRENGTH: f32 = 4000.0;

/// Sword swing arc radius.
pub(super) const SWORD_ARC_RADIUS: f32 = 60.0 * UNIT_SCALE;

/// Sword swing damage.
pub(super) const SWORD_DAMAGE: f32 = 40.0;

/// Sword swing cooldown in seconds.
pub(super) const SWORD_COOLDOWN: f32 = 0.6;

/// Total lifetime of the sword arc visual (grow + fade window).
pub(super) const SWORD_ARC_DURATION: f32 = 0.28;

/// How long the arc takes to expand from a point at the avatar to its full
/// `SWORD_ARC_RADIUS`. Kept short so the strike reads as a fast slash.
pub(super) const SWORD_ARC_GROW_DURATION: f32 = 0.05;

/// Half-angle (radians) of the swing arc — the strip spans 2× this around
/// the swing direction. Damage hits enemies inside this angular wedge.
pub(super) const SWORD_ARC_HALF_ANGLE: f32 = std::f32::consts::FRAC_PI_3; // 60°

/// Radial thickness of the arc strip (between inner edge and outer edge).
pub(super) const SWORD_ARC_THICKNESS: f32 = 16.0 * UNIT_SCALE;

/// Number of angular subdivisions in the arc strip mesh — higher = smoother
/// curve at the cost of more triangles.
pub(super) const SWORD_ARC_SEGMENTS: u32 = 28;

/// Minimum non-zero scale used for the arc Transform during the grow
/// animation. Keeps the entity visible (a literal zero scale would discard
/// it) and the mesh-bounds calculation stable.
pub(super) const SWORD_ARC_MIN_SCALE: f32 = 0.001;

/// Melee damage multiplier applied to the swordcerer avatar (0.25 = takes
/// 25% incoming melee damage, i.e. 75% reduction).
pub(super) const AVATAR_MELEE_DAMAGE_MULTIPLIER: f32 = 0.25;

/// Lifetime of the per-unit `HitFlash` flag set when the sword arc hits a
/// target. Independent of the overlay's own fade duration.
pub(super) const SWORD_HIT_FLASH_DURATION: f32 = 0.12;

/// Hitbox radius for the swordcerer avatar.
pub(crate) const AVATAR_HITBOX_RADIUS: f32 = 8.0 * UNIT_SCALE;

/// Hitbox height for the swordcerer avatar.
pub(crate) const AVATAR_HITBOX_HEIGHT: f32 = 25.0 * UNIT_SCALE;

/// WASD acceleration strength for player-controlled movement.
pub(super) const PLAYER_ACCELERATION: f32 = 2000.0;

/// Velocity damping for the player avatar (higher = snappier feel).
pub(super) const PLAYER_DAMPING: f32 = 0.75;

/// Velocity impulse applied toward the cursor on sword swing.
pub(super) const SWORD_LUNGE_SPEED: f32 = 600.0;

/// Bottom margin for the "Enter the Fray" button.
pub(super) const FRAY_BUTTON_BOTTOM: f32 = 30.0;

/// Font size for the "Enter the Fray" button text.
pub(super) const FRAY_BUTTON_FONT_SIZE: f32 = 18.0;

/// Button text when idle.
pub(super) const FRAY_BUTTON_TEXT: &str = "Enter the Fray";

/// Button text when choosing location.
pub(super) const FRAY_BUTTON_TEXT_CHOOSING: &str = "Where?";

/// Distance threshold for Close Call achievement (XZ distance from wizard).
pub(crate) const CLOSE_CALL_DISTANCE: f32 = 800.0;
