use crate::game::constants::UNIT_SCALE;

/// 160x16 sprite sheet: 10 columns of 16x16 pixels.
pub(super) const FLORA_SPRITE_COUNT: usize = 10;

pub(super) const FLORA_SPRITE_WIDTH: f32 = 18.0 * UNIT_SCALE;
pub(super) const FLORA_SPRITE_HEIGHT: f32 = 18.0 * UNIT_SCALE;

pub(crate) const FLORA_COUNT: u32 = 80;

pub(super) const FLORA_TRAMPLE_RADIUS: f32 = 20.0;
pub(super) const FLORA_TRAMPLE_RADIUS_SQ: f32 = FLORA_TRAMPLE_RADIUS * FLORA_TRAMPLE_RADIUS;

pub(super) const FLORA_MIN_X: f32 = -2200.0;
pub(super) const FLORA_MAX_X: f32 = 2400.0;
pub(super) const FLORA_MIN_Z: f32 = -2200.0;
pub(super) const FLORA_MAX_Z: f32 = 2200.0;

/// Shadow scale for flora sprites.
pub(super) const FLORA_SHADOW_SCALE: f32 = 0.4;
