use bevy::prelude::*;

use crate::game::constants::UNIT_SCALE;

/// Movement speed for the battlemage avatar (8x normal infantry speed).
pub(super) const AVATAR_MOVEMENT_SPEED: f32 = 920.0;

/// Health of the battlemage avatar on the field.
pub(super) const AVATAR_HEALTH: f32 = 150.0;

/// Sprite tint for the battlemage avatar (neutral white — swordcerer has its own sprite sheet).
pub(super) const AVATAR_SPRITE_TINT: Color = Color::WHITE;

pub(super) use crate::game::units::constants::DEFAULT_SPRITE_HEIGHT as AVATAR_SPRITE_HEIGHT;
/// Sprite dimensions for the battlemage avatar.
pub(super) use crate::game::units::constants::DEFAULT_SPRITE_WIDTH as AVATAR_SPRITE_WIDTH;

/// Magic missile damage from the battlemage avatar.
pub(super) const MISSILE_DAMAGE: f32 = 20.0;

/// Mana cost per magic missile fired from the field.
pub(super) const MISSILE_MANA_COST: f32 = 8.0;

/// Magic missile cooldown in seconds.
pub(super) const MISSILE_COOLDOWN: f32 = 0.25;

/// Magic missile speed.
pub(super) const MISSILE_SPEED: f32 = 800.0;

/// Sword swing arc radius.
pub(super) const SWORD_ARC_RADIUS: f32 = 60.0 * UNIT_SCALE;

/// Sword swing damage.
pub(super) const SWORD_DAMAGE: f32 = 40.0;

/// Sword swing cooldown in seconds.
pub(super) const SWORD_COOLDOWN: f32 = 0.6;

/// Sword swing arc duration (flash time).
pub(super) const SWORD_ARC_DURATION: f32 = 0.15;

/// Hitbox radius for the battlemage avatar.
pub(super) const AVATAR_HITBOX_RADIUS: f32 = 8.0 * UNIT_SCALE;

/// Hitbox height for the battlemage avatar.
pub(super) const AVATAR_HITBOX_HEIGHT: f32 = 25.0 * UNIT_SCALE;

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
