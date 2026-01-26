use bevy::prelude::*;

use super::super::super::components::{PrimedSpell, Spell};

/// Spell configuration for Raise The Dead
pub const PRIMED_RAISE_THE_DEAD: PrimedSpell = PrimedSpell {
    spell: Spell::RaiseTheDead,
    cast_time: 1.0, // 1 second cast time
};

/// Initial interval between resurrections (in seconds)
pub const INITIAL_CHANNEL_INTERVAL: f32 = 0.8;

/// Minimum interval between resurrections after ramp-up (in seconds)
pub const MIN_CHANNEL_INTERVAL: f32 = 0.3;

/// Time it takes to ramp from initial to minimum interval (in seconds)
pub const CHANNEL_RAMP_TIME: f32 = 4.0;

/// Mana cost per resurrected corpse
pub const MANA_COST_PER_CORPSE: f32 = 10.0;

/// Radius around cursor to search for corpses (in world units)
pub const RESURRECTION_RADIUS: f32 = 150.0;

/// Color for undead units (bright green)
pub const UNDEAD_COLOR: Color = Color::srgb(0.3, 0.8, 0.4);
