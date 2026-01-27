//! Constants for the Teleport spell.

use bevy::prelude::*;

use crate::game::units::wizard::components::PrimedSpell;
use crate::game::units::wizard::components::Spell;

/// Primed Teleport spell configuration.
pub const PRIMED_TELEPORT: PrimedSpell = PrimedSpell {
    spell: Spell::Teleport,
    cast_time: 1.0, // First cast time (destination placement)
};

/// Second cast time for source circle and teleportation.
pub const SECOND_CAST_TIME: f32 = 2.0;

/// Mana cost for teleportation (charged on second cast).
pub const MANA_COST: f32 = 20.0;

/// Radius of both destination and source circles.
pub const CIRCLE_RADIUS: f32 = 150.0;

/// Color for destination circle (light blue, low opacity).
pub const DESTINATION_COLOR: Color = Color::srgba(0.0, 0.6, 1.0, 0.25);

/// Color for source circle during second cast (brighter blue).
pub const SOURCE_COLOR: Color = Color::srgba(0.0, 0.8, 1.0, 0.35);

/// Scale threshold at which pulse animation begins (prevents pulsing during growth).
pub const PULSE_THRESHOLD: f32 = 0.9;
