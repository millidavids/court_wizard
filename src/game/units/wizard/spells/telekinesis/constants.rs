use crate::game::units::wizard::components::{PrimedSpell, Spell};

/// PrimedSpell constant for Telekinesis.
pub const PRIMED_TELEKINESIS: PrimedSpell = PrimedSpell {
    spell: Spell::Telekinesis,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

/// Cast time for Telekinesis in seconds (very short).
pub const CAST_TIME: f32 = 0.5;

/// Mana cost for casting Telekinesis.
pub const MANA_COST: f32 = 5.0;

/// Maximum distance from cursor to find a nearby drop (units).
pub const PICKUP_RADIUS: f32 = 100.0;

/// Y position of the indicator ring (slightly above ground).
pub const INDICATOR_Y: f32 = 2.0;

/// Radius of the indicator ring around the targeted drop.
pub const INDICATOR_RADIUS: f32 = 15.0;

/// Color of the telekinesis targeting indicator.
pub const INDICATOR_COLOR: bevy::prelude::Color = bevy::prelude::Color::srgba(0.6, 0.9, 1.0, 0.7);
