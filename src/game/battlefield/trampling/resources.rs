use bevy::prelude::*;

use super::constants::TRAMPLING_CELL_SIZE;
use crate::config::save_data::SavedTrampling;
use crate::game::constants::BATTLEFIELD_SIZE;

/// Tracks trampling intensity per battlefield tile.
///
/// Each cell stores a 0.0–1.0 value representing how trampled the ground is.
/// Persists across levels (decays between battles) but resets on exit to menu.
#[derive(Resource)]
pub struct TramplingGrid {
    /// Trampling intensity per tile (row-major, 0.0 = pristine, 1.0 = fully trampled).
    pub values: Vec<f32>,
    /// Number of tiles per side of the battlefield.
    pub tiles_per_side: usize,
    /// Whether any value changed since last texture sync.
    pub dirty: bool,
}

impl TramplingGrid {
    /// Creates a new zeroed trampling grid sized to cover the battlefield.
    pub fn new() -> Self {
        let tiles_per_side = (BATTLEFIELD_SIZE / TRAMPLING_CELL_SIZE) as usize;
        Self {
            values: vec![0.0; tiles_per_side * tiles_per_side],
            tiles_per_side,
            dirty: false,
        }
    }

    /// Converts a world XZ position to a flat tile index.
    /// Returns None if outside the battlefield bounds.
    pub fn world_to_index(&self, x: f32, z: f32) -> Option<usize> {
        let half = BATTLEFIELD_SIZE / 2.0;
        let col = ((x + half) / TRAMPLING_CELL_SIZE).floor() as isize;
        let row = ((z + half) / TRAMPLING_CELL_SIZE).floor() as isize;
        let side = self.tiles_per_side as isize;

        if col < 0 || row < 0 || col >= side || row >= side {
            return None;
        }

        Some(row as usize * self.tiles_per_side + col as usize)
    }

    /// Subtracts `amount` from every cell, clamping to 0.0 (grass grows back).
    pub fn decay(&mut self, amount: f32) {
        let mut changed = false;
        for v in &mut self.values {
            if *v > 0.0 {
                *v = (*v - amount).max(0.0);
                changed = true;
            }
        }
        if changed {
            self.dirty = true;
        }
    }

    /// Resets all values to zero (full exit to menu).
    pub fn reset(&mut self) {
        for v in &mut self.values {
            *v = 0.0;
        }
        self.dirty = true;
    }

    /// Exports non-zero cells as a sparse save format.
    pub fn to_saved(&self) -> SavedTrampling {
        let cells: Vec<(u32, u8)> = self
            .values
            .iter()
            .enumerate()
            .filter(|(_, v)| **v > 0.0)
            .map(|(i, v)| (i as u32, (*v * 255.0) as u8))
            .collect();
        SavedTrampling {
            grid_size: self.tiles_per_side,
            cells,
        }
    }

    /// Restores from saved data. Ignores if grid size doesn't match.
    pub fn from_saved(&mut self, saved: &SavedTrampling) {
        if saved.grid_size != self.tiles_per_side || saved.cells.is_empty() {
            return;
        }
        for &(idx, intensity) in &saved.cells {
            let i = idx as usize;
            if i < self.values.len() {
                self.values[i] = intensity as f32 / 255.0;
            }
        }
        self.dirty = true;
    }
}
