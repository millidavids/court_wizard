//! Flow field pathfinding implementation.
//!
//! Uses Dijkstra's algorithm to generate vector fields that guide units toward goals
//! while avoiding obstacles and respecting terrain costs.

use bevy::prelude::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// A flow field that guides units toward a goal position.
///
/// Each cell contains a direction vector and a cost value.
#[derive(Clone)]
pub struct FlowField {
    /// Direction to move from each cell (normalized Vec3, Y=0).
    pub directions: Vec<Vec3>,
    /// Movement cost for each cell (1.0 = normal, 3.0 = mud, f32::INFINITY = blocked).
    pub costs: Vec<f32>,
    /// Integration field - pathfinding distance from each cell to goal.
    pub integration: Vec<f32>,
    /// Grid width (number of cells in X direction).
    pub width: usize,
    /// Grid height (number of cells in Z direction).
    pub height: usize,
}

/// Priority queue node for Dijkstra's algorithm.
#[derive(Copy, Clone)]
struct Node {
    cost: f32,
    x: usize,
    z: usize,
}

// Priority queue ordering: lowest cost first
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for Node {}

impl FlowField {
    /// Creates a new empty flow field with the specified dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            directions: vec![Vec3::ZERO; size],
            costs: vec![1.0; size],
            integration: vec![f32::INFINITY; size],
            width,
            height,
        }
    }

    /// Gets the 1D index for a 2D grid position.
    #[inline]
    fn index(&self, x: usize, z: usize) -> usize {
        z * self.width + x
    }

    /// Marks cells as blocked (impassable).
    #[allow(dead_code)]
    pub fn mark_blocked(&mut self, cells: &[(usize, usize)]) {
        for &(x, z) in cells {
            if x < self.width && z < self.height {
                let idx = self.index(x, z);
                self.costs[idx] = f32::INFINITY;
            }
        }
    }

    /// Sets terrain cost for cells (higher = slower movement).
    ///
    /// This replaces the current cost - use this when clearing obstacles or
    /// updating terrain. For obstacles that overlap, the last one wins.
    #[allow(dead_code)]
    pub fn set_terrain_cost(&mut self, cells: &[(usize, usize)], cost: f32) {
        for &(x, z) in cells {
            if x < self.width && z < self.height {
                let idx = self.index(x, z);
                self.costs[idx] = cost;
            }
        }
    }

    /// Generates the flow field using Dijkstra's algorithm from a goal position.
    ///
    /// # Arguments
    ///
    /// * `goal_x` - Goal cell X coordinate
    /// * `goal_z` - Goal cell Z coordinate
    /// * `satisfaction_radius_cells` - Radius in cells where units are "close enough" (0 = no satisfaction radius)
    pub fn generate(&mut self, goal_x: usize, goal_z: usize, satisfaction_radius_cells: usize) {
        // Initialize integration field (cost to reach goal from each cell)
        let mut integration = vec![f32::INFINITY; self.width * self.height];

        // Goal has zero cost
        let goal_idx = self.index(goal_x, goal_z);
        integration[goal_idx] = 0.0;

        // Priority queue for Dijkstra
        let mut queue = BinaryHeap::new();
        queue.push(Node {
            cost: 0.0,
            x: goal_x,
            z: goal_z,
        });

        // Dijkstra's algorithm with diagonal corner-cutting prevention
        while let Some(Node { cost, x, z }) = queue.pop() {
            let current_idx = self.index(x, z);

            // Skip if we've already found a better path
            if cost > integration[current_idx] {
                continue;
            }

            // Cardinal neighbors (always checked)
            let cardinals = [
                (x.wrapping_sub(1), z, 1.0), // West
                (x + 1, z, 1.0),              // East
                (x, z.wrapping_sub(1), 1.0),  // South
                (x, z + 1, 1.0),              // North
            ];

            // Track which cardinals are passable for diagonal gating
            let w_pass = x.wrapping_sub(1) < self.width
                && !self.costs[self.index(x.wrapping_sub(1), z)].is_infinite();
            let e_pass =
                x + 1 < self.width && !self.costs[self.index(x + 1, z)].is_infinite();
            let s_pass = z.wrapping_sub(1) < self.height
                && !self.costs[self.index(x, z.wrapping_sub(1))].is_infinite();
            let n_pass =
                z + 1 < self.height && !self.costs[self.index(x, z + 1)].is_infinite();

            // Diagonal neighbors: only expand if both adjacent cardinals are passable
            let diagonals: [(usize, usize, f32, bool); 4] = [
                (x.wrapping_sub(1), z.wrapping_sub(1), 1.414, w_pass && s_pass), // SW
                (x + 1, z.wrapping_sub(1), 1.414, e_pass && s_pass),             // SE
                (x.wrapping_sub(1), z + 1, 1.414, w_pass && n_pass),             // NW
                (x + 1, z + 1, 1.414, e_pass && n_pass),                         // NE
            ];

            // Process cardinal neighbors
            for (nx, nz, distance_mult) in cardinals {
                if nx >= self.width || nz >= self.height {
                    continue;
                }
                let neighbor_idx = self.index(nx, nz);
                let terrain_cost = self.costs[neighbor_idx];
                if terrain_cost.is_infinite() {
                    continue;
                }
                let new_cost = cost + terrain_cost * distance_mult;
                if new_cost < integration[neighbor_idx] {
                    integration[neighbor_idx] = new_cost;
                    queue.push(Node {
                        cost: new_cost,
                        x: nx,
                        z: nz,
                    });
                }
            }

            // Process diagonal neighbors (gated by adjacent cardinals)
            for (nx, nz, distance_mult, passable) in diagonals {
                if !passable || nx >= self.width || nz >= self.height {
                    continue;
                }
                let neighbor_idx = self.index(nx, nz);
                let terrain_cost = self.costs[neighbor_idx];
                if terrain_cost.is_infinite() {
                    continue;
                }
                let new_cost = cost + terrain_cost * distance_mult;
                if new_cost < integration[neighbor_idx] {
                    integration[neighbor_idx] = new_cost;
                    queue.push(Node {
                        cost: new_cost,
                        x: nx,
                        z: nz,
                    });
                }
            }
        }

        // Generate flow field from integration field (before moving)
        self.generate_flow_from_integration(
            &integration,
            goal_x,
            goal_z,
            satisfaction_radius_cells,
        );

        // Smooth directions using Line Integral Convolution for organic movement
        self.smooth_with_lic();

        // Store integration field for pathfinding distance queries (moved, not cloned)
        self.integration = integration;
    }

    /// Generates direction vectors from integration costs.
    fn generate_flow_from_integration(
        &mut self,
        integration: &[f32],
        goal_x: usize,
        goal_z: usize,
        satisfaction_radius: usize,
    ) {
        for z in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, z);

                // Skip blocked or unreachable cells
                if self.costs[idx].is_infinite() || integration[idx].is_infinite() {
                    self.directions[idx] = Vec3::ZERO;
                    continue;
                }

                // Check if within satisfaction radius of goal (if radius > 0)
                if satisfaction_radius > 0 {
                    let dx = (x as isize - goal_x as isize).unsigned_abs();
                    let dz = (z as isize - goal_z as isize).unsigned_abs();
                    let distance_squared = dx * dx + dz * dz;
                    let radius_squared = satisfaction_radius * satisfaction_radius;

                    if distance_squared <= radius_squared {
                        // Within satisfaction radius - no need to move
                        self.directions[idx] = Vec3::ZERO;
                        continue;
                    }
                }

                // Find the neighbor with lowest integration cost
                let mut best_direction = Vec3::ZERO;
                let mut best_cost = integration[idx];

                // Cardinal neighbors (always checked)
                let cardinals = [
                    (x.wrapping_sub(1), z, Vec3::new(-1.0, 0.0, 0.0)), // West
                    (x + 1, z, Vec3::new(1.0, 0.0, 0.0)),              // East
                    (x, z.wrapping_sub(1), Vec3::new(0.0, 0.0, -1.0)), // South
                    (x, z + 1, Vec3::new(0.0, 0.0, 1.0)),              // North
                ];

                for (nx, nz, direction) in cardinals {
                    if nx >= self.width || nz >= self.height {
                        continue;
                    }
                    let neighbor_cost = integration[self.index(nx, nz)];
                    if neighbor_cost < best_cost {
                        best_cost = neighbor_cost;
                        best_direction = direction;
                    }
                }

                // Track which cardinals are passable for diagonal gating
                let w_pass = x.wrapping_sub(1) < self.width
                    && !self.costs[self.index(x.wrapping_sub(1), z)].is_infinite();
                let e_pass =
                    x + 1 < self.width && !self.costs[self.index(x + 1, z)].is_infinite();
                let s_pass = z.wrapping_sub(1) < self.height
                    && !self.costs[self.index(x, z.wrapping_sub(1))].is_infinite();
                let n_pass =
                    z + 1 < self.height && !self.costs[self.index(x, z + 1)].is_infinite();

                // Diagonal neighbors: only consider if both adjacent cardinals are passable
                let diag_neighbors = [
                    (x.wrapping_sub(1), z.wrapping_sub(1), Vec3::new(-1.0, 0.0, -1.0), w_pass && s_pass), // SW
                    (x + 1, z.wrapping_sub(1), Vec3::new(1.0, 0.0, -1.0), e_pass && s_pass),              // SE
                    (x.wrapping_sub(1), z + 1, Vec3::new(-1.0, 0.0, 1.0), w_pass && n_pass),              // NW
                    (x + 1, z + 1, Vec3::new(1.0, 0.0, 1.0), e_pass && n_pass),                           // NE
                ];

                for (nx, nz, direction, passable) in diag_neighbors {
                    if !passable || nx >= self.width || nz >= self.height {
                        continue;
                    }
                    let neighbor_cost = integration[self.index(nx, nz)];
                    if neighbor_cost < best_cost {
                        best_cost = neighbor_cost;
                        best_direction = direction;
                    }
                }

                // Normalize direction (diagonals are longer)
                self.directions[idx] = best_direction.normalize_or_zero();
            }
        }
    }

    /// Smooths directions near obstacles using Line Integral Convolution.
    ///
    /// Only applied to cells with elevated costs (near walls/hazards) where the
    /// bilinear gradient fallback in `sample()` uses stored cell directions.
    /// Open-terrain cells already get smooth directions from gradient interpolation.
    ///
    /// For each eligible cell, traces a streamline forward and backward through
    /// the vector field, averaging directions along the path. Tracing stops at
    /// grid edges, blocked cells, or zero-direction cells.
    fn smooth_with_lic(&mut self) {
        const LIC_STEPS: usize = 3;
        const STEP_SIZE: f32 = 0.5;

        // Skip entirely if no cells have elevated costs (no obstacles to smooth around).
        if !self.costs.iter().any(|&c| c > 1.0 && !c.is_infinite()) {
            return;
        }

        let original_directions = self.directions.clone();
        let width = self.width;
        let height = self.height;

        for z in 0..height {
            for x in 0..width {
                let idx = z * width + x;

                // Skip blocked/unreachable/zero-direction cells
                if self.costs[idx].is_infinite() || original_directions[idx] == Vec3::ZERO {
                    continue;
                }

                // Only smooth near obstacles (elevated cost). Open-terrain cells
                // (cost == 1.0) get smooth directions from gradient interpolation.
                if self.costs[idx] <= 1.0 {
                    continue;
                }

                let mut accumulated = original_directions[idx];
                let mut total_weight = 1.0_f32;

                // Trace in both directions: forward (+1) and backward (-1)
                for &sign in &[1.0_f32, -1.0_f32] {
                    let mut pos_x = x as f32 + 0.5;
                    let mut pos_z = z as f32 + 0.5;

                    for step in 0..LIC_STEPS {
                        // Get direction at current continuous position
                        let cell_x = pos_x.floor() as isize;
                        let cell_z = pos_z.floor() as isize;

                        if cell_x < 0
                            || cell_z < 0
                            || cell_x >= width as isize
                            || cell_z >= height as isize
                        {
                            break;
                        }

                        let cell_idx = cell_z as usize * width + cell_x as usize;

                        if self.costs[cell_idx].is_infinite()
                            || original_directions[cell_idx] == Vec3::ZERO
                        {
                            break;
                        }

                        let dir = original_directions[cell_idx];

                        // Advance position along the streamline
                        pos_x += dir.x * STEP_SIZE * sign;
                        pos_z += dir.z * STEP_SIZE * sign;

                        // Weight decreases with distance from center
                        let weight = 1.0 / (1.0 + (step + 1) as f32);
                        accumulated += dir * weight;
                        total_weight += weight;
                    }
                }

                self.directions[idx] = (accumulated / total_weight).normalize_or_zero();
            }
        }
    }

    /// Returns the direction at a grid cell, or `Vec3::ZERO` if out of bounds.
    #[inline]
    fn sample_cell(&self, x: isize, z: isize) -> Vec3 {
        if x < 0 || z < 0 || x >= self.width as isize || z >= self.height as isize {
            return Vec3::ZERO;
        }
        self.directions[z as usize * self.width + x as usize]
    }

    /// Returns the integration cost at a grid cell, or `f32::INFINITY` if out of bounds.
    #[inline]
    fn sample_integration_cost(&self, x: isize, z: isize) -> f32 {
        if x < 0 || z < 0 || x >= self.width as isize || z >= self.height as isize {
            return f32::INFINITY;
        }
        self.integration[z as usize * self.width + x as usize]
    }

    /// Samples the flow field at a world position using gradient-based interpolation.
    ///
    /// Computes the negative gradient of the bilinear integration cost surface for
    /// smooth, continuous direction vectors. Falls back to the stored cell direction
    /// when any surrounding cell has INFINITY cost (near obstacles/boundaries).
    pub fn sample(&self, world_pos: Vec3, world_min: Vec2, cell_size: f32) -> Vec3 {
        // Convert world position to continuous grid coordinates (cell centers at +0.5)
        let gx = (world_pos.x - world_min.x) / cell_size - 0.5;
        let gz = (world_pos.z - world_min.y) / cell_size - 0.5;

        let x0 = gx.floor() as isize;
        let z0 = gz.floor() as isize;
        let x1 = x0 + 1;
        let z1 = z0 + 1;

        // Get integration costs of surrounding cells
        let c00 = self.sample_integration_cost(x0, z0);
        let c10 = self.sample_integration_cost(x1, z0);
        let c01 = self.sample_integration_cost(x0, z1);
        let c11 = self.sample_integration_cost(x1, z1);

        // If any cost is INFINITY (near obstacle/boundary), fall back to stored
        // cell direction to avoid gradient artifacts pointing into walls
        if c00.is_infinite() || c10.is_infinite() || c01.is_infinite() || c11.is_infinite() {
            // Use the direction of the cell the unit is currently in
            let cell_x = ((world_pos.x - world_min.x) / cell_size).floor() as isize;
            let cell_z = ((world_pos.z - world_min.y) / cell_size).floor() as isize;
            return self.sample_cell(cell_x, cell_z);
        }

        // Compute negative gradient of the bilinear integration cost surface.
        // This gives a smooth, continuous direction that always points downhill
        // toward the goal, eliminating cell-boundary discontinuities.
        let fx = gx - x0 as f32;
        let fz = gz - z0 as f32;

        let dx = (1.0 - fz) * (c10 - c00) + fz * (c11 - c01);
        let dz = (1.0 - fx) * (c01 - c00) + fx * (c11 - c10);

        Vec3::new(-dx, 0.0, -dz).normalize_or_zero()
    }

    /// Samples the pathfinding distance at a world position.
    ///
    /// Returns the integration field cost (pathfinding distance to goal),
    /// or f32::INFINITY if out of bounds or unreachable.
    pub fn sample_distance(&self, world_pos: Vec3, world_min: Vec2, cell_size: f32) -> f32 {
        // Convert world position to grid coordinates
        let grid_x = ((world_pos.x - world_min.x) / cell_size).floor() as isize;
        let grid_z = ((world_pos.z - world_min.y) / cell_size).floor() as isize;

        // Check bounds
        if grid_x < 0
            || grid_z < 0
            || grid_x >= self.width as isize
            || grid_z >= self.height as isize
        {
            return f32::INFINITY;
        }

        let idx = self.index(grid_x as usize, grid_z as usize);
        self.integration[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_field_generation() {
        // Create a 10x10 grid
        let mut field = FlowField::new(10, 10);

        // Goal in center (5, 5), no satisfaction radius
        field.generate(5, 5, 0);

        // Check that corner points toward center
        let world_min = Vec2::new(0.0, 0.0);
        let cell_size = 1.0;

        // Sample from corner (0, 0) - should point toward (5, 5)
        let direction = field.sample(Vec3::new(0.5, 0.0, 0.5), world_min, cell_size);

        // Direction should have positive X and Z components
        assert!(direction.x > 0.0);
        assert!(direction.z > 0.0);
        assert_eq!(direction.y, 0.0);
    }

    #[test]
    fn test_blocked_cells() {
        let mut field = FlowField::new(10, 10);

        // Block a wall from (5, 0) to (5, 9)
        for z in 0..10 {
            field.mark_blocked(&[(5, z)]);
        }

        field.generate(7, 5, 0); // Goal on right side of wall

        // Sample from left side (3, 5) - should path around wall
        let world_min = Vec2::new(0.0, 0.0);
        let cell_size = 1.0;
        let direction = field.sample(Vec3::new(3.5, 0.0, 5.5), world_min, cell_size);

        // Should not point directly at goal (blocked), should go around
        // Direction should point north or south to go around wall
        assert!(direction.z.abs() > 0.1);
    }

    #[test]
    fn test_satisfaction_radius() {
        let mut field = FlowField::new(10, 10);

        // Goal at (5, 5) with satisfaction radius of 2 cells
        field.generate(5, 5, 2);

        let world_min = Vec2::new(0.0, 0.0);
        let cell_size = 1.0;

        // Sample from within satisfaction radius (4, 5) - should be zero
        let direction = field.sample(Vec3::new(4.5, 0.0, 5.5), world_min, cell_size);
        assert_eq!(direction, Vec3::ZERO);

        // Sample from outside satisfaction radius (0, 0) - should have direction
        let direction = field.sample(Vec3::new(0.5, 0.0, 0.5), world_min, cell_size);
        assert!(direction.length() > 0.0);
    }
}
