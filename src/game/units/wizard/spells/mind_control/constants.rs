use bevy::prelude::*;

use crate::game::units::wizard::components::{PrimedSpell, Spell};

/// PrimedSpell constant for Mind Control.
pub const PRIMED_MIND_CONTROL: PrimedSpell = PrimedSpell {
    spell: Spell::MindControl,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

/// Cast time (seconds).
pub const CAST_TIME: f32 = 1.0;

/// Mana cost per cast.
pub const MANA_COST: f32 = 50.0;

/// Cooldown between casts (seconds).
pub const COOLDOWN: f32 = 2.0;

/// Duration of mind control effect (seconds away from caster before wearing off).
pub const EFFECT_WEAR_OFF_DURATION: f32 = 10.0;

/// Maximum number of mind-controlled units at once (wizard spell).
pub const MAX_CONTROLLED: u32 = 3;

/// Max distance from cursor to find a target during cast.
pub const TARGET_SEARCH_RADIUS: f32 = 80.0;

/// Purple highlight tint for the targeted unit during cast.
pub const HIGHLIGHT_COLOR: Color = Color::srgb(0.7, 0.2, 1.0);
