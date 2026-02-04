//! Pathfinding resources.

use bevy::prelude::*;

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
    pub world_max: Vec2,

    /// Flow field for attackers (flows toward King).
    pub attacker_field: FlowField,
    /// Flow field for king defenders (closest to action).
    pub king_defender_field: FlowField,
    /// Flow field for infantry defenders (middle line).
    pub infantry_defender_field: FlowField,
    /// Flow field for archer defenders (back line).
    pub archer_defender_field: FlowField,

    /// Last known King position (for detecting significant movement).
    pub last_king_pos: Vec2,

    /// Pending rebuild requests (processed asynchronously).
    pub pending_rebuilds: Vec<RebuildRequest>,
}

/// Request to rebuild a flow field region.
#[derive(Clone)]
pub struct RebuildRequest {
    /// Which field(s) to rebuild.
    pub field_type: RebuildFieldType,
    /// Goal position for the field.
    pub goal_pos: Vec2,
    /// Cells that need rebuilding (empty = full rebuild).
    pub dirty_cells: Vec<(usize, usize)>,
}

/// Specifies which field(s) to rebuild.
#[derive(Clone, Copy, Debug)]
pub enum RebuildFieldType {
    /// Rebuild attacker field only.
    Attacker,
    /// Rebuild king defender field only.
    KingDefender,
    /// Rebuild infantry defender field only.
    InfantryDefender,
    /// Rebuild archer defender field only.
    ArcherDefender,
    /// Rebuild all defender fields.
    AllDefenders,
    /// Rebuild all fields.
    All,
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

        let attacker_field = FlowField::new(grid_width, grid_height);
        let king_defender_field = FlowField::new(grid_width, grid_height);
        let infantry_defender_field = FlowField::new(grid_width, grid_height);
        let archer_defender_field = FlowField::new(grid_width, grid_height);

        Self {
            cell_size,
            grid_width,
            grid_height,
            world_min,
            world_max,
            attacker_field,
            king_defender_field,
            infantry_defender_field,
            archer_defender_field,
            last_king_pos: Vec2::ZERO,
            pending_rebuilds: Vec::new(),
        }
    }

    /// Converts a world position to grid coordinates.
    ///
    /// Returns None if the position is outside the grid bounds.
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

    /// Initializes flow fields with default goals.
    ///
    /// Should be called once at startup after the resource is created.
    ///
    /// # Arguments
    ///
    /// * `king_pos` - King's starting position
    /// * `wizard_pos` - Wizard position (reference point for defender rally points)
    /// * `king_rally_dist` - Distance from wizard for king rally point
    /// * `infantry_rally_dist` - Distance from wizard for infantry rally point
    /// * `archer_rally_dist` - Distance from wizard for archer rally point
    /// * `king_radius` - Satisfaction radius for king (in cells)
    /// * `infantry_radius` - Satisfaction radius for infantry (in cells)
    /// * `archer_radius` - Satisfaction radius for archers (in cells)
    pub fn initialize_fields(
        &mut self,
        king_pos: Vec2,
        wizard_pos: Vec2,
        king_rally_dist: f32,
        infantry_rally_dist: f32,
        archer_rally_dist: f32,
        king_radius: usize,
        infantry_radius: usize,
        archer_radius: usize,
    ) {
        // Convert world positions to grid coordinates
        if let Some((king_x, king_z)) = self.world_to_grid(king_pos) {
            self.attacker_field.generate(king_x, king_z);
            self.last_king_pos = king_pos;
        }

        // Calculate rally points radially from wizard toward battlefield center
        // Direction: toward origin (0, 0) from wizard position
        let to_center = Vec2::ZERO - wizard_pos;
        let direction = to_center.normalize();

        let king_rally = wizard_pos + direction * king_rally_dist;
        let infantry_rally = wizard_pos + direction * infantry_rally_dist;
        let archer_rally = wizard_pos + direction * archer_rally_dist;

        if let Some((x, z)) = self.world_to_grid(king_rally) {
            self.king_defender_field
                .generate_with_radius(x, z, king_radius);
        }

        if let Some((x, z)) = self.world_to_grid(infantry_rally) {
            self.infantry_defender_field
                .generate_with_radius(x, z, infantry_radius);
        }

        if let Some((x, z)) = self.world_to_grid(archer_rally) {
            self.archer_defender_field
                .generate_with_radius(x, z, archer_radius);
        }
    }
}
