//! Shared spawn helpers for unit cell positioning.

use rand::Rng;

/// Generates a random position within a tight spread around a cell center.
/// Used for spawning units randomly within their assigned grid cell.
/// Returns (x, z) coordinates offset from the center point.
pub(crate) fn random_position_in_cell(rng: &mut impl Rng, cell_x: f32, cell_z: f32) -> (f32, f32) {
    use crate::game::constants::GRID_ROW_DEPTH;
    let spread = GRID_ROW_DEPTH / 8.0;
    let final_x = cell_x + rng.random_range(-spread..spread);
    let final_z = cell_z + rng.random_range(-spread..spread);
    (final_x, final_z)
}
