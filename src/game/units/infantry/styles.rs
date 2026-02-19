use bevy::prelude::*;

use crate::game::constants::{ATTACKER_BASE, DEFENDER_BASE, TINT_RED, tint};

// Entity Colors — derived from team base with subtle tints
pub const DEFENDER_COLOR: Color = DEFENDER_BASE;
pub const ATTACKER_COLOR: Color = ATTACKER_BASE;
pub const KINGS_GUARD_COLOR: Color = tint(DEFENDER_BASE, TINT_RED, 0.3); // Red tint (elite)

// Entity Sizes
pub const UNIT_RADIUS: f32 = 8.0; // Circle radius for units
