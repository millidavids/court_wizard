use bevy::prelude::*;

use crate::game::constants::STONE_COLOR_DARK;

/// Base reddish-brown color for trampled ground overlays (derived from stone underlay).
pub(super) const TRAMPLING_COLOR: Color = STONE_COLOR_DARK;

/// Maximum alpha the trampling overlay can reach.
pub(super) const TRAMPLING_MAX_ALPHA: f32 = 0.8;

/// How much trampling intensity increases each time a unit enters a tile.
pub(super) const TRAMPLING_INCREMENT: f32 = 0.03;

/// How much trampling intensity decays between levels (grass grows back).
pub(super) const TRAMPLING_DECAY_PER_LEVEL: f32 = 0.1;

/// How often (seconds) the overlay texture syncs with the trampling grid.
pub(super) const TRAMPLING_SYNC_INTERVAL: f32 = 0.25;

/// Y offset above the ground plane for the trampling overlay quad.
/// Must be just above ground tiles (Y=0) but below unit meshes.
pub(super) const TRAMPLING_OVERLAY_Y: f32 = 0.1;

/// World size of each trampling grid cell (matches pathfinding cell size).
pub(in crate::game) const TRAMPLING_CELL_SIZE: f32 = 10.0;
