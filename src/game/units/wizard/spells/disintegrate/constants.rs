//! Disintegrate spell constants.

use bevy::prelude::*;

/// Damage dealt per tick to entities in the beam.
pub const DAMAGE_PER_TICK: f32 = 5.0;

/// Time between damage ticks (in seconds).
pub const DAMAGE_INTERVAL: f32 = 0.1;

/// Width of the beam for both collision detection and visual rendering.
pub const BEAM_WIDTH: f32 = 10.0;

/// Color of the beam (green).
pub const BEAM_COLOR: Color = Color::srgb(0.0, 1.0, 0.0);

/// Mana cost per second while channeling.
pub const MANA_COST_PER_SECOND: f32 = 20.0;

/// Beam length (extends through the battlefield).
pub const BEAM_LENGTH: f32 = 5000.0;

/// Height offset from wizard position where beam originates.
pub const BEAM_ORIGIN_HEIGHT_OFFSET: f32 = 100.0;

/// Cast time before beam activates (in seconds).
pub const CAST_TIME: f32 = 1.5;

/// Time for beam to grow to full length (in seconds).
pub const BEAM_GROWTH_TIME: f32 = 0.2;
