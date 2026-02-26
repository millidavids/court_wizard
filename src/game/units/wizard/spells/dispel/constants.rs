use bevy::prelude::*;

use crate::game::units::wizard::components::{PrimedSpell, Spell};

/// PrimedSpell constant for Dispel.
pub const PRIMED_DISPEL: PrimedSpell = PrimedSpell {
    spell: Spell::Dispel,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

/// Cast time (instant).
pub const CAST_TIME: f32 = 0.0;

/// Mana cost per cast.
pub const MANA_COST: f32 = 5.0;

/// Cooldown between casts (seconds).
pub const COOLDOWN: f32 = 0.5;

/// Speed of the dispel projectile (units/second).
pub const PROJECTILE_SPEED: f32 = 1600.0;

/// Visual radius of the projectile circle.
pub const PROJECTILE_RADIUS: f32 = 5.0;

/// Color of the dispel projectile and impact sphere.
pub const PROJECTILE_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);

/// How long the impact sphere lasts (seconds).
pub const IMPACT_DURATION: f32 = 0.5;

/// How fast the impact sphere expands (units/second in radius).
pub const IMPACT_EXPAND_SPEED: f32 = 200.0;

/// Starting alpha of the impact sphere.
pub const IMPACT_INITIAL_ALPHA: f32 = 0.3;

/// Height offset above spell origin for spawning the projectile.
pub const SPAWN_HEIGHT_OFFSET: f32 = 0.0;

/// Maximum projectile lifetime before auto-despawn (seconds).
pub const PROJECTILE_LIFETIME: f32 = 3.0;
