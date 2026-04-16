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
    /// Each non-zero cell is packed as 5 bytes (u32 LE index + u8 intensity),
    /// then base64-encoded — ~5.6× smaller in TOML than one-line-per-cell.
    pub fn to_saved(&self) -> SavedTrampling {
        let non_zero = self.values.iter().filter(|v| **v > 0.0).count();
        let mut packed: Vec<u8> = Vec::with_capacity(non_zero * 5);
        for (i, v) in self.values.iter().enumerate() {
            if *v > 0.0 {
                packed.extend_from_slice(&(i as u32).to_le_bytes());
                packed.push((*v * 255.0) as u8);
            }
        }
        SavedTrampling {
            grid_size: self.tiles_per_side,
            cells_b64: crate::config::save_data::to_base64(&packed),
        }
    }

    /// Restores from saved data. Ignores if grid size doesn't match.
    pub fn restore_saved(&mut self, saved: &SavedTrampling) {
        if saved.grid_size != self.tiles_per_side || saved.cells_b64.is_empty() {
            return;
        }
        let Some(packed) = crate::config::save_data::from_base64(&saved.cells_b64) else {
            return;
        };
        let mut wrote_any = false;
        for chunk in packed.chunks_exact(5) {
            let idx = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as usize;
            let intensity = chunk[4];
            if idx < self.values.len() {
                self.values[idx] = intensity as f32 / 255.0;
                wrote_any = true;
            }
        }
        if wrote_any {
            self.dirty = true;
        }
    }
}
