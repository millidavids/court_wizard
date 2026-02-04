//! Pathfinding resources.

use bevy::prelude::*;
use bevy::tasks::Task;

use super::flow_field::FlowField;

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

    /// Creates a new flow field with the base costs applied.
    pub fn create_field_with_base_costs(&self) -> FlowField {
        let mut field = FlowField::new(self.grid_width, self.grid_height);
        field.costs = self.base_costs.clone();
        field
    }
}
