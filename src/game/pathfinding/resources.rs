//! Pathfinding resources.

use std::collections::VecDeque;

use bevy::prelude::*;
use bevy::tasks::Task;

use super::flow_field::FlowField;

/// Which flow field to rebuild.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebuildTarget {
    Attacker,
    Defender,
}

/// Grid-based pathfinding resource that manages flow fields for different teams.
///
/// Flow fields are pre-computed vector fields that guide units toward their goals
/// while avoiding obstacles.
#[derive(Resource)]
pub struct PathfindingGrid {
    /// Size of each grid cell in world units.
    pub cell_size: f32,
    /// Number of cells in X direction.
    pub grid_width: usize,
    /// Number of cells in Z direction.
    pub grid_height: usize,
    /// Minimum world coordinates (bottom-left corner).
    pub world_min: Vec2,
    /// Maximum world coordinates (top-right corner).
    #[allow(dead_code)]
    pub world_max: Vec2,

    /// Flow field for attackers (flows toward King), None until first generation.
    pub attacker_field: Option<FlowField>,
    /// Pending async rebuild task for attacker field.
    pub pending_attacker_rebuild: Option<Task<FlowField>>,

    /// Flow field for defenders (flows toward King's target), None when not activated.
    pub defender_field: Option<FlowField>,
    /// Pending async rebuild task for defender field.
    pub pending_defender_rebuild: Option<Task<FlowField>>,

    /// Last known King position (for detecting significant movement).
    pub last_king_pos: Vec2,
    /// King's current target entity (None = not activated yet).
    pub king_current_target: Option<Entity>,

    /// Base terrain costs template (copied for each field generation).
    /// This stores obstacles like walls that affect all fields.
    pub base_costs: Vec<f32>,

    /// True when `base_costs` changed while a rebuild was already in progress.
    /// When the pending rebuild completes it will be stale, so a fresh rebuild
    /// is triggered immediately.
    pub costs_dirty: bool,

    /// Debounce timer for obstacle changes. When > 0, a rebuild is pending but
    /// deferred to batch rapid changes (e.g. wall placements) into one rebuild.
    pub rebuild_debounce: f32,

    /// Delay before rebuilding the defender field toward spawn when enemies disappear.
    /// Prevents oscillation when enemies die rapidly and new ones appear quickly.
    pub defender_rally_delay: f32,

    /// Last position the defender field was built toward.
    /// Used to avoid rebuilds when the target entity changes but the position is similar.
    pub last_defender_target_pos: Vec2,

    /// Queue of pending flow field rebuilds. Only one is processed per frame
    /// to spread the cost and avoid frame spikes.
    pub rebuild_queue: VecDeque<RebuildTarget>,
}

impl PathfindingGrid {
    /// Creates a new pathfinding grid covering the battlefield.
    ///
    /// # Arguments
    ///
    /// * `battlefield_size` - Size of the battlefield (assuming square, centered at origin)
    /// * `cell_size` - Size of each grid cell in world units
    pub fn new(battlefield_size: f32, cell_size: f32) -> Self {
        let half_size = battlefield_size / 2.0;
        let world_min = Vec2::new(-half_size, -half_size);
        let world_max = Vec2::new(half_size, half_size);

        let grid_width = (battlefield_size / cell_size).ceil() as usize;
        let grid_height = grid_width; // Square grid

        let base_costs = vec![1.0; grid_width * grid_height];

        Self {
            cell_size,
            grid_width,
            grid_height,
            world_min,
            world_max,
            attacker_field: None,
            pending_attacker_rebuild: None,
            defender_field: None,
            pending_defender_rebuild: None,
            last_king_pos: Vec2::ZERO,
            king_current_target: None,
            base_costs,
            costs_dirty: false,
            rebuild_debounce: 0.0,
            defender_rally_delay: 0.0,
            last_defender_target_pos: Vec2::ZERO,
            rebuild_queue: VecDeque::new(),
        }
    }

    /// Converts a world position to grid coordinates.
    ///
    /// Returns None if the position is outside the grid bounds.
    #[allow(dead_code)]
    pub fn world_to_grid(&self, world_pos: Vec2) -> Option<(usize, usize)> {
        let grid_x = ((world_pos.x - self.world_min.x) / self.cell_size).floor() as isize;
        let grid_z = ((world_pos.y - self.world_min.y) / self.cell_size).floor() as isize;

        if grid_x < 0
            || grid_z < 0
            || grid_x >= self.grid_width as isize
            || grid_z >= self.grid_height as isize
        {
            return None;
        }

        Some((grid_x as usize, grid_z as usize))
    }

    /// Converts a rectangular world bounds to a list of grid cells.
    ///
    /// Used for marking obstacles that span multiple cells.
    pub fn world_bounds_to_cells(&self, bounds: Rect) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();

        // Convert bounds to grid coordinates
        let min_x = ((bounds.min.x - self.world_min.x) / self.cell_size)
            .floor()
            .max(0.0) as usize;
        let min_z = ((bounds.min.y - self.world_min.y) / self.cell_size)
            .floor()
            .max(0.0) as usize;
        let max_x = ((bounds.max.x - self.world_min.x) / self.cell_size)
            .ceil()
            .min(self.grid_width as f32) as usize;
        let max_z = ((bounds.max.y - self.world_min.y) / self.cell_size)
            .ceil()
            .min(self.grid_height as f32) as usize;

        // Collect all cells within bounds
        for x in min_x..max_x {
            for z in min_z..max_z {
                cells.push((x, z));
            }
        }

        cells
    }

    /// Gets the 1D index for a 2D grid position.
    #[inline]
    fn index(&self, x: usize, z: usize) -> usize {
        z * self.grid_width + x
    }

    /// Marks cells as blocked in the base cost template.
    pub fn mark_blocked(&mut self, cells: &[(usize, usize)]) {
        for &(x, z) in cells {
            if x < self.grid_width && z < self.grid_height {
                let idx = self.index(x, z);
                self.base_costs[idx] = f32::INFINITY;
            }
        }
    }

    /// Sets terrain cost for cells in the base cost template.
    pub fn set_terrain_cost(&mut self, cells: &[(usize, usize)], cost: f32) {
        for &(x, z) in cells {
            if x < self.grid_width && z < self.grid_height {
                let idx = self.index(x, z);
                self.base_costs[idx] = cost;
            }
        }
    }

    /// Samples the base terrain cost at a world position.
    ///
    /// Returns the cost of the cell at the given position, or 1.0 if out of bounds.
    pub fn sample_base_cost(&self, world_pos: Vec3) -> f32 {
        let grid_x = ((world_pos.x - self.world_min.x) / self.cell_size).floor() as isize;
        let grid_z = ((world_pos.z - self.world_min.y) / self.cell_size).floor() as isize;

        if grid_x < 0
            || grid_z < 0
            || grid_x >= self.grid_width as isize
            || grid_z >= self.grid_height as isize
        {
            return 1.0;
        }

        let idx = self.index(grid_x as usize, grid_z as usize);
        self.base_costs[idx]
    }

    /// Creates a new flow field with the base costs applied.
    ///
    /// Cells adjacent to blocked (wall) cells are inflated to a higher cost so the
    /// flow field steers units slightly away from walls instead of hugging them.
    /// This prevents units from sliding along wall edges and catching on corners.
    pub fn create_field_with_base_costs(&self) -> FlowField {
        let mut field = FlowField::new(self.grid_width, self.grid_height);
        field.costs = self.base_costs.clone();

        // Inflate costs near blocked cells so the flow field avoids wall edges
        const WALL_PROXIMITY_COST: f32 = 4.0;
        let w = self.grid_width;
        let h = self.grid_height;

        for z in 0..h {
            for x in 0..w {
                let idx = z * w + x;
                if !self.base_costs[idx].is_infinite() {
                    continue;
                }
                // This cell is blocked — inflate passable neighbors
                for dz in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dz == 0 {
                            continue;
                        }
                        let nx = x as i32 + dx;
                        let nz = z as i32 + dz;
                        if nx >= 0 && nz >= 0 && (nx as usize) < w && (nz as usize) < h {
                            let ni = nz as usize * w + nx as usize;
                            // Only inflate normal-cost cells (don't reduce hazards or re-block)
                            if field.costs[ni] < WALL_PROXIMITY_COST {
                                field.costs[ni] = WALL_PROXIMITY_COST;
                            }
                        }
                    }
                }
            }
        }

        field
    }

    /// Enqueues a rebuild target, skipping if it's already queued.
    pub fn enqueue_rebuild(&mut self, target: RebuildTarget) {
        if !self.rebuild_queue.contains(&target) {
            self.rebuild_queue.push_back(target);
        }
    }
}
