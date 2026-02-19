use bevy::prelude::*;

use crate::game::constants::{ATTACKER_BASE, DEFENDER_BASE, TINT_WHITE, tint};

// Archer unit colors (50% lighter than infantry to distinguish at a glance)
pub const DEFENDER_ARCHER_COLOR: Color = tint(DEFENDER_BASE, TINT_WHITE, 0.5); // 50% toward white
pub const ATTACKER_ARCHER_COLOR: Color = tint(ATTACKER_BASE, TINT_WHITE, 0.5); // 50% toward white

// Arrow
pub const ARROW_COLOR: Color = Color::srgb(0.45, 0.3, 0.15); // Brown
pub const ARCHER_RADIUS: f32 = 8.0; // Same as infantry
