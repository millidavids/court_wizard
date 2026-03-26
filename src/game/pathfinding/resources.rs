//! Pathfinding resources.

use bevy::prelude::*;
use bevy::tasks::Task;

use super::flow_field::FlowField;

/// Grid-based pathfinding resource that manages flow fields for different teams.
///
/// Flow fields are pre-computed vector fields that guide units toward their goals
/// while avoiding obstacles. All fields are continuously rebuilt in parallel on
/// background threads.
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

    /// Flow field for assassins (flows toward archer center of mass, avoids infantry).
    pub assassin_field: Option<FlowField>,
    /// Pending async rebuild task for assassin field.
    pub pending_assassin_rebuild: Option<Task<FlowField>>,
    /// Last known assassin target position (archer center of mass).
    pub last_assassin_target_pos: Vec2,

    /// Flow field for staging attackers (flows toward the staging point).
    /// Built once at initialization and never rebuilt.
    pub staging_field: Option<FlowField>,

    /// Last known King position.
    pub last_king_pos: Vec2,
    /// King's current target entity (None = no enemies / not activated yet).
    pub king_current_target: Option<Entity>,

    /// Base terrain costs template (copied for each field generation).
    /// This stores obstacles like walls that affect all fields.
    pub base_costs: Vec<f32>,

    /// Delay before rebuilding the defender field toward spawn when enemies disappear.
    /// Prevents oscillation when enemies die rapidly and new ones appear quickly.
    pub defender_rally_delay: f32,

    /// Last position the defender field was built toward.
    pub last_defender_target_pos: Vec2,
}

impl PathfindingGrid {
    /// Creates a new pathfinding grid covering the battlefield.
    ///
    /// # Arguments
    ///
    /// * `battlefield_size` - Size of the battlefield (square, centered at origin)
    /// * `cell_size` - Size of each grid cell in world units
    /// * `x_extension` - Extra world units to extend in the +X direction (for spawn area behind wall)
    pub fn new(battlefield_size: f32, cell_size: f32, x_extension: f32) -> Self {
        let half_size = battlefield_size / 2.0;
        let world_min = Vec2::new(-half_size, -half_size);
        let world_max = Vec2::new(half_size + x_extension, half_size);

        let grid_width = ((world_max.x - world_min.x) / cell_size).ceil() as usize;
        let grid_height = ((world_max.y - world_min.y) / cell_size).ceil() as usize;

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
            assassin_field: None,
            pending_assassin_rebuild: None,
            last_assassin_target_pos: Vec2::ZERO,
            staging_field: None,
            last_king_pos: Vec2::ZERO,
            king_current_target: None,
            base_costs,
            defender_rally_delay: 0.0,
            last_defender_target_pos: Vec2::ZERO,
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

    /// Creates a new flow field with the base costs applied, using a distance
    /// transform to inflate costs around obstacles.
    ///
    /// Distance 0 = blocked (INFINITY). Distance 1 = hard block (INFINITY) to
    /// prevent unit centers from being adjacent to walls. Distance 2-3 = elevated
    /// costs to steer units away from walls. This prevents wall-hugging,
    /// corner-clipping, and units getting stuck in narrow passages.
    pub fn create_field_with_base_costs(&self) -> FlowField {
        /// Cost applied to cells at Chebyshev distance 1 from a wall.
        /// Very high but NOT INFINITY so units still get flow directions if they
        /// wander into this zone (e.g. pushed by combat or flocking near corners).
        const INFLATION_COST_DISTANCE_1: f32 = 20.0;
        /// Cost applied to cells at Chebyshev distance 2 from a wall.
        const INFLATION_COST_DISTANCE_2: f32 = 6.0;
        /// Cost applied to cells at Chebyshev distance 3 from a wall.
        const INFLATION_COST_DISTANCE_3: f32 = 3.0;

        let mut field = FlowField::new(self.grid_width, self.grid_height);
        field.costs = self.base_costs.clone();

        let w = self.grid_width;
        let h = self.grid_height;

        // Build Chebyshev distance field via two-pass distance transform.
        // 0 = blocked cell, u16::MAX = far from any wall.
        let mut distance: Vec<u16> = self
            .base_costs
            .iter()
            .map(|c| if c.is_infinite() { 0 } else { u16::MAX })
            .collect();

        // Forward pass (top-left to bottom-right)
        for z in 0..h {
            for x in 0..w {
                let idx = z * w + x;
                if distance[idx] == 0 {
                    continue;
                }
                let mut d = distance[idx];
                // Cardinal: left, up
                if x > 0 {
                    d = d.min(distance[idx - 1].saturating_add(1));
                }
                if z > 0 {
                    d = d.min(distance[idx - w].saturating_add(1));
                }
                // Diagonal: top-left, top-right
                if x > 0 && z > 0 {
                    d = d.min(distance[idx - w - 1].saturating_add(1));
                }
                if x + 1 < w && z > 0 {
                    d = d.min(distance[idx - w + 1].saturating_add(1));
                }
                distance[idx] = d;
            }
        }

        // Backward pass (bottom-right to top-left)
        for z in (0..h).rev() {
            for x in (0..w).rev() {
                let idx = z * w + x;
                if distance[idx] == 0 {
                    continue;
                }
                let mut d = distance[idx];
                // Cardinal: right, down
                if x + 1 < w {
                    d = d.min(distance[idx + 1].saturating_add(1));
                }
                if z + 1 < h {
                    d = d.min(distance[idx + w].saturating_add(1));
                }
                // Diagonal: bottom-left, bottom-right
                if x > 0 && z + 1 < h {
                    d = d.min(distance[idx + w - 1].saturating_add(1));
                }
                if x + 1 < w && z + 1 < h {
                    d = d.min(distance[idx + w + 1].saturating_add(1));
                }
                distance[idx] = d;
            }
        }

        // Apply inflation from distance field
        for (d, cost) in distance.iter().zip(field.costs.iter_mut()) {
            match *d {
                0 => {} // Already INFINITY (blocked)
                1 if *cost < INFLATION_COST_DISTANCE_1 => {
                    *cost = INFLATION_COST_DISTANCE_1;
                }
                2 if *cost < INFLATION_COST_DISTANCE_2 => {
                    *cost = INFLATION_COST_DISTANCE_2;
                }
                3 if *cost < INFLATION_COST_DISTANCE_3 => {
                    *cost = INFLATION_COST_DISTANCE_3;
                }
                _ => {}
            }
        }

        field
    }

    /// Returns cells within `bounds` that intersect the given shape.
    /// Broadphase AABB followed by per-cell narrowphase shape test.
    pub fn shape_filtered_cells(
        &self,
        bounds: Rect,
        shape: &super::messages::ObstacleShape,
    ) -> Vec<(usize, usize)> {
        let cell_half = self.cell_size * 0.5;
        let world_min = self.world_min;
        let cell_size = self.cell_size;
        self.world_bounds_to_cells(bounds)
            .into_iter()
            .filter(|&(x, z)| {
                let cx = world_min.x + (x as f32 + 0.5) * cell_size;
                let cz = world_min.y + (z as f32 + 0.5) * cell_size;
                shape.intersects_cell(Vec2::new(cx, cz), cell_half)
            })
            .collect()
    }

}
