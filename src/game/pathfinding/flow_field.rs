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
                (x + 1, z, 1.0),             // East
                (x, z.wrapping_sub(1), 1.0), // South
                (x, z + 1, 1.0),             // North
            ];

            // Track which cardinals are passable for diagonal gating
            let w_pass = x.wrapping_sub(1) < self.width
                && !self.costs[self.index(x.wrapping_sub(1), z)].is_infinite();
            let e_pass = x + 1 < self.width && !self.costs[self.index(x + 1, z)].is_infinite();
            let s_pass = z.wrapping_sub(1) < self.height
                && !self.costs[self.index(x, z.wrapping_sub(1))].is_infinite();
            let n_pass = z + 1 < self.height && !self.costs[self.index(x, z + 1)].is_infinite();

            // Diagonal neighbors: only expand if both adjacent cardinals are passable
            let diagonals: [(usize, usize, f32, bool); 4] = [
                (
                    x.wrapping_sub(1),
                    z.wrapping_sub(1),
                    1.414,
                    w_pass && s_pass,
                ), // SW
                (x + 1, z.wrapping_sub(1), 1.414, e_pass && s_pass), // SE
                (x.wrapping_sub(1), z + 1, 1.414, w_pass && n_pass), // NW
                (x + 1, z + 1, 1.414, e_pass && n_pass),             // NE
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

        // Store integration field (needed by gradient computation and distance queries)
        self.integration = integration;

        // Generate smooth directions: gradient-based for open terrain, 8-neighbor + LIC near obstacles
        self.generate_smooth_directions(goal_x, goal_z, satisfaction_radius_cells);
    }

    /// Generates smooth direction vectors from the integration field.
    ///
    /// Uses bilinear gradient of integration costs for continuous directions in open
    /// terrain. Falls back to 8-neighbor best-direction + LIC smoothing near obstacles
    /// where gradient computation would be corrupted by INFINITY costs.
    fn generate_smooth_directions(
        &mut self,
        goal_x: usize,
        goal_z: usize,
        satisfaction_radius: usize,
    ) {
        let width = self.width;
        let height = self.height;

        // First pass: compute directions for every cell
        for z in 0..height {
            for x in 0..width {
                let idx = self.index(x, z);

                // Skip blocked or unreachable cells
                if self.costs[idx].is_infinite() || self.integration[idx].is_infinite() {
                    self.directions[idx] = Vec3::ZERO;
                    continue;
                }

                // Check if within satisfaction radius of goal
                if satisfaction_radius > 0 {
                    let dx = (x as isize - goal_x as isize).unsigned_abs();
                    let dz = (z as isize - goal_z as isize).unsigned_abs();
                    if dx * dx + dz * dz <= satisfaction_radius * satisfaction_radius {
                        self.directions[idx] = Vec3::ZERO;
                        continue;
                    }
                }

                // Try bilinear gradient first (smooth, continuous directions).
                // This requires all 4 surrounding integration costs to be finite.
                let x0 = x as isize - 1;
                let z0 = z as isize - 1;
                let x1 = x as isize;
                let z1 = z as isize;

                let c00 = self.sample_integration_cost(x0, z0);
                let c10 = self.sample_integration_cost(x1, z0);
                let c01 = self.sample_integration_cost(x0, z1);
                let c11 = self.sample_integration_cost(x1, z1);

                if !c00.is_infinite()
                    && !c10.is_infinite()
                    && !c01.is_infinite()
                    && !c11.is_infinite()
                {
                    // Gradient of the bilinear surface at cell center (fx=fz=0.5)
                    let dx = 0.5 * (c10 - c00) + 0.5 * (c11 - c01);
                    let dz = 0.5 * (c01 - c00) + 0.5 * (c11 - c10);
                    let dir = Vec3::new(-dx, 0.0, -dz).normalize_or_zero();
                    if dir != Vec3::ZERO {
                        self.directions[idx] = dir;
                        continue;
                    }
                }

                // Fallback: 8-neighbor best-direction (near obstacles/boundaries)
                self.directions[idx] = self.best_neighbor_direction(x, z);
            }
        }

        // Second pass: LIC smoothing for cells near obstacles
        self.smooth_with_lic();
    }

    /// Finds the neighbor with lowest integration cost and returns the direction to it.
    fn best_neighbor_direction(&self, x: usize, z: usize) -> Vec3 {
        let idx = self.index(x, z);
        let mut best_direction = Vec3::ZERO;
        let mut best_cost = self.integration[idx];

        let cardinals = [
            (x.wrapping_sub(1), z, Vec3::new(-1.0, 0.0, 0.0)),
            (x + 1, z, Vec3::new(1.0, 0.0, 0.0)),
            (x, z.wrapping_sub(1), Vec3::new(0.0, 0.0, -1.0)),
            (x, z + 1, Vec3::new(0.0, 0.0, 1.0)),
        ];

        for (nx, nz, direction) in cardinals {
            if nx >= self.width || nz >= self.height {
                continue;
            }
            let neighbor_cost = self.integration[self.index(nx, nz)];
            if neighbor_cost < best_cost {
                best_cost = neighbor_cost;
                best_direction = direction;
            }
        }

        let w_pass = x.wrapping_sub(1) < self.width
            && !self.costs[self.index(x.wrapping_sub(1), z)].is_infinite();
        let e_pass = x + 1 < self.width && !self.costs[self.index(x + 1, z)].is_infinite();
        let s_pass = z.wrapping_sub(1) < self.height
            && !self.costs[self.index(x, z.wrapping_sub(1))].is_infinite();
        let n_pass = z + 1 < self.height && !self.costs[self.index(x, z + 1)].is_infinite();

        let diag_neighbors = [
            (
                x.wrapping_sub(1),
                z.wrapping_sub(1),
                Vec3::new(-1.0, 0.0, -1.0),
                w_pass && s_pass,
            ),
            (
                x + 1,
                z.wrapping_sub(1),
                Vec3::new(1.0, 0.0, -1.0),
                e_pass && s_pass,
            ),
            (
                x.wrapping_sub(1),
                z + 1,
                Vec3::new(-1.0, 0.0, 1.0),
                w_pass && n_pass,
            ),
            (x + 1, z + 1, Vec3::new(1.0, 0.0, 1.0), e_pass && n_pass),
        ];

        for (nx, nz, direction, passable) in diag_neighbors {
            if !passable || nx >= self.width || nz >= self.height {
                continue;
            }
            let neighbor_cost = self.integration[self.index(nx, nz)];
            if neighbor_cost < best_cost {
                best_cost = neighbor_cost;
                best_direction = direction;
            }
        }

        best_direction.normalize_or_zero()
    }

    /// Smooths directions near obstacles using Line Integral Convolution.
    ///
    /// Only applied to cells that used the 8-neighbor fallback (near obstacles),
    /// identified by having elevated terrain costs.
    fn smooth_with_lic(&mut self) {
        const LIC_STEPS: usize = 3;
        const STEP_SIZE: f32 = 0.5;

        if !self.costs.iter().any(|&c| c > 1.0 && !c.is_infinite()) {
            return;
        }

        let original_directions = self.directions.clone();
        let width = self.width;
        let height = self.height;

        for z in 0..height {
            for x in 0..width {
                let idx = z * width + x;

                if self.costs[idx].is_infinite() || original_directions[idx] == Vec3::ZERO {
                    continue;
                }

                // Only smooth cells with elevated cost (near obstacles)
                if self.costs[idx] <= 1.0 {
                    continue;
                }

                let mut accumulated = original_directions[idx];
                let mut total_weight = 1.0_f32;

                for &sign in &[1.0_f32, -1.0_f32] {
                    let mut pos_x = x as f32 + 0.5;
                    let mut pos_z = z as f32 + 0.5;

                    for step in 0..LIC_STEPS {
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
                        pos_x += dir.x * STEP_SIZE * sign;
                        pos_z += dir.z * STEP_SIZE * sign;

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

    /// Samples the flow field at a world position.
    ///
    /// Returns the precomputed smooth direction for the cell containing the position.
    /// Directions are computed during `generate()` using bilinear gradient interpolation
    /// in open terrain and 8-neighbor + LIC smoothing near obstacles.
    pub fn sample(&self, world_pos: Vec3, world_min: Vec2, cell_size: f32) -> Vec3 {
        let cell_x = ((world_pos.x - world_min.x) / cell_size).floor() as isize;
        let cell_z = ((world_pos.z - world_min.y) / cell_size).floor() as isize;
        self.sample_cell(cell_x, cell_z)
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
