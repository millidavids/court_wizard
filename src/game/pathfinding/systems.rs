//! Pathfinding systems.

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};

use crate::game::constants::BATTLEFIELD_SIZE;
use crate::game::units::king::components::King;

use super::events::{ObstacleChanged, ObstacleType};
use super::flow_field::FlowField;
use super::resources::{PathfindingGrid, RebuildFieldType, RebuildRequest};

/// Cell size for the pathfinding grid (in world units).
const CELL_SIZE: f32 = 25.0;

/// Wizard position (XZ plane) - the reference point for defender formations.
const WIZARD_POS: Vec2 = Vec2::new(-1425.0, 1550.0);

/// Defender rally points at different distances from wizard (radial distance).
/// Archers closest to wizard (back line), Infantry in middle, King furthest (front line).
/// Distances measured radially from wizard toward battlefield center.
const ARCHER_RALLY_DISTANCE: f32 = 800.0; // Back line for ranged attacks (closest to wizard)
const INFANTRY_RALLY_DISTANCE: f32 = 1400.0; // Front line, closest to action (furthest from wizard)
const KING_RALLY_DISTANCE: f32 = 1100.0; // Middle line, closest to action (medium from wizard)

/// Satisfaction radii for each defender type (in grid cells).
/// With 25-unit cells: Archer=20 cells (500 units), Infantry=16 cells (400 units), King=12 cells (300 units)
const ARCHER_SATISFACTION_RADIUS: usize = 20;
const INFANTRY_SATISFACTION_RADIUS: usize = 16;
const KING_SATISFACTION_RADIUS: usize = 12;

/// Initializes the pathfinding grid and generates initial flow fields.
pub fn initialize_pathfinding(mut commands: Commands, king_query: Query<&Transform, With<King>>) {
    // Create the pathfinding grid
    let mut pathfinding = PathfindingGrid::new(BATTLEFIELD_SIZE, CELL_SIZE);

    // Get King's starting position
    let king_pos = if let Ok(king_transform) = king_query.single() {
        Vec2::new(king_transform.translation.x, king_transform.translation.z)
    } else {
        // Fallback if King hasn't spawned yet
        WIZARD_POS
    };

    // Initialize flow fields with layered rally points
    pathfinding.initialize_fields(
        king_pos,
        WIZARD_POS,
        KING_RALLY_DISTANCE,
        INFANTRY_RALLY_DISTANCE,
        ARCHER_RALLY_DISTANCE,
        KING_SATISFACTION_RADIUS,
        INFANTRY_SATISFACTION_RADIUS,
        ARCHER_SATISFACTION_RADIUS,
    );

    info!(
        "Pathfinding initialized: {}x{} grid ({} cells)",
        pathfinding.grid_width,
        pathfinding.grid_height,
        pathfinding.grid_width * pathfinding.grid_height
    );

    // Insert as resource
    commands.insert_resource(pathfinding);
}

/// Handles obstacle change events and updates flow fields accordingly.
pub fn handle_obstacle_events(
    mut obstacle_events: MessageReader<ObstacleChanged>,
    mut pathfinding: ResMut<PathfindingGrid>,
) {
    for event in obstacle_events.read() {
        // Convert world bounds to grid cells
        let affected_cells = pathfinding.world_bounds_to_cells(event.bounds);

        match event.obstacle_type {
            ObstacleType::Blocked => {
                // Mark cells as blocked (infinite cost)
                pathfinding.attacker_field.mark_blocked(&affected_cells);
                pathfinding
                    .king_defender_field
                    .mark_blocked(&affected_cells);
                pathfinding
                    .infantry_defender_field
                    .mark_blocked(&affected_cells);
                pathfinding
                    .archer_defender_field
                    .mark_blocked(&affected_cells);

                debug!(
                    "Blocked {} cells at ({}, {}) size {}x{}",
                    affected_cells.len(),
                    event.bounds.min.x,
                    event.bounds.min.y,
                    event.bounds.width(),
                    event.bounds.height()
                );
            }
            ObstacleType::SlowTerrain(multiplier) => {
                // Set cells to slow terrain cost
                pathfinding
                    .attacker_field
                    .set_terrain_cost(&affected_cells, multiplier);
                pathfinding
                    .king_defender_field
                    .set_terrain_cost(&affected_cells, multiplier);
                pathfinding
                    .infantry_defender_field
                    .set_terrain_cost(&affected_cells, multiplier);
                pathfinding
                    .archer_defender_field
                    .set_terrain_cost(&affected_cells, multiplier);

                debug!(
                    "Set {} cells to slow terrain ({}x) at ({}, {})",
                    affected_cells.len(),
                    multiplier,
                    event.bounds.min.x,
                    event.bounds.min.y
                );
            }
            ObstacleType::Removed => {
                // Reset cells to normal terrain cost (1.0)
                pathfinding
                    .attacker_field
                    .set_terrain_cost(&affected_cells, 1.0);
                pathfinding
                    .king_defender_field
                    .set_terrain_cost(&affected_cells, 1.0);
                pathfinding
                    .infantry_defender_field
                    .set_terrain_cost(&affected_cells, 1.0);
                pathfinding
                    .archer_defender_field
                    .set_terrain_cost(&affected_cells, 1.0);

                debug!(
                    "Removed obstacle from {} cells at ({}, {})",
                    affected_cells.len(),
                    event.bounds.min.x,
                    event.bounds.min.y
                );
            }
        }

        // Queue rebuild request for all fields when obstacles change
        let king_pos = pathfinding.last_king_pos;
        pathfinding.pending_rebuilds.push(RebuildRequest {
            field_type: RebuildFieldType::All,
            goal_pos: king_pos, // Will be updated for each field
            dirty_cells: affected_cells,
        });
    }
}

/// Component attached to entities with async rebuild tasks.
#[derive(Component)]
pub(super) struct FlowFieldRebuildTask {
    task: Task<FlowField>,
    field_type: RebuildFieldType,
}

/// Spawns async tasks to rebuild flow fields.
pub fn spawn_rebuild_tasks(mut commands: Commands, mut pathfinding: ResMut<PathfindingGrid>) {
    if pathfinding.pending_rebuilds.is_empty() {
        return;
    }

    let task_pool = AsyncComputeTaskPool::get();

    // Extract values before draining to avoid borrow checker issues
    let attacker_field = pathfinding.attacker_field.clone();
    let king_defender_field = pathfinding.king_defender_field.clone();
    let infantry_defender_field = pathfinding.infantry_defender_field.clone();
    let archer_defender_field = pathfinding.archer_defender_field.clone();
    let king_pos = pathfinding.last_king_pos;
    let world_min = pathfinding.world_min;
    let cell_size = pathfinding.cell_size;

    // Calculate defender rally points
    let to_center = Vec2::ZERO - WIZARD_POS;
    let direction = to_center.normalize();
    let king_rally = WIZARD_POS + direction * KING_RALLY_DISTANCE;
    let infantry_rally = WIZARD_POS + direction * INFANTRY_RALLY_DISTANCE;
    let archer_rally = WIZARD_POS + direction * ARCHER_RALLY_DISTANCE;

    for rebuild in pathfinding.pending_rebuilds.drain(..) {
        use RebuildFieldType::*;

        // Determine which fields to rebuild
        let rebuild_attacker = matches!(rebuild.field_type, Attacker | All);
        let rebuild_king = matches!(rebuild.field_type, KingDefender | AllDefenders | All);
        let rebuild_infantry = matches!(rebuild.field_type, InfantryDefender | AllDefenders | All);
        let rebuild_archer = matches!(rebuild.field_type, ArcherDefender | AllDefenders | All);

        // Rebuild attacker field (flows toward King)
        if rebuild_attacker {
            spawn_field_rebuild_task(
                &mut commands,
                &task_pool,
                attacker_field.clone(),
                king_pos,
                world_min,
                cell_size,
                RebuildFieldType::Attacker,
                8, // Default satisfaction radius for attackers
            );
        }

        // Rebuild king defender field
        if rebuild_king {
            spawn_field_rebuild_task(
                &mut commands,
                &task_pool,
                king_defender_field.clone(),
                king_rally,
                world_min,
                cell_size,
                RebuildFieldType::KingDefender,
                KING_SATISFACTION_RADIUS,
            );
        }

        // Rebuild infantry defender field
        if rebuild_infantry {
            spawn_field_rebuild_task(
                &mut commands,
                &task_pool,
                infantry_defender_field.clone(),
                infantry_rally,
                world_min,
                cell_size,
                RebuildFieldType::InfantryDefender,
                INFANTRY_SATISFACTION_RADIUS,
            );
        }

        // Rebuild archer defender field
        if rebuild_archer {
            spawn_field_rebuild_task(
                &mut commands,
                &task_pool,
                archer_defender_field.clone(),
                archer_rally,
                world_min,
                cell_size,
                RebuildFieldType::ArcherDefender,
                ARCHER_SATISFACTION_RADIUS,
            );
        }
    }
}

/// Helper function to spawn a single field rebuild task.
fn spawn_field_rebuild_task(
    commands: &mut Commands,
    task_pool: &AsyncComputeTaskPool,
    mut field: FlowField,
    goal_pos: Vec2,
    world_min: Vec2,
    cell_size: f32,
    field_type: RebuildFieldType,
    satisfaction_radius: usize,
) {
    let task = task_pool.spawn(async move {
        // Convert goal to grid coordinates
        let goal_x = ((goal_pos.x - world_min.x) / cell_size).floor() as usize;
        let goal_z = ((goal_pos.y - world_min.y) / cell_size).floor() as usize;

        // Rebuild the field with satisfaction radius
        field.generate_with_radius(goal_x, goal_z, satisfaction_radius);
        field
    });

    commands.spawn(FlowFieldRebuildTask { task, field_type });
}

/// Applies completed rebuild tasks to the pathfinding grid.
pub fn apply_completed_rebuilds(
    mut commands: Commands,
    mut pathfinding: ResMut<PathfindingGrid>,
    mut rebuild_tasks: Query<(Entity, &mut FlowFieldRebuildTask)>,
) {
    for (entity, mut task) in &mut rebuild_tasks {
        // Check if task is complete (non-blocking poll)
        if let Some(new_field) = bevy::tasks::block_on(bevy::tasks::poll_once(&mut task.task)) {
            // Swap in the new field
            match task.field_type {
                RebuildFieldType::Attacker => {
                    pathfinding.attacker_field = new_field;
                    debug!("Applied rebuilt attacker flow field");
                }
                RebuildFieldType::KingDefender => {
                    pathfinding.king_defender_field = new_field;
                    debug!("Applied rebuilt king defender flow field");
                }
                RebuildFieldType::InfantryDefender => {
                    pathfinding.infantry_defender_field = new_field;
                    debug!("Applied rebuilt infantry defender flow field");
                }
                RebuildFieldType::ArcherDefender => {
                    pathfinding.archer_defender_field = new_field;
                    debug!("Applied rebuilt archer defender flow field");
                }
                RebuildFieldType::AllDefenders | RebuildFieldType::All => {
                    // These shouldn't appear in individual tasks
                    warn!(
                        "Unexpected {:?} field type in rebuild task",
                        task.field_type
                    );
                }
            }

            // Despawn the task entity
            commands.entity(entity).despawn();
        }
    }
}

/// Updates King position and rebuilds attacker flow field when he moves significantly.
pub fn update_king_position(
    mut pathfinding: ResMut<PathfindingGrid>,
    king_query: Query<&Transform, With<King>>,
) {
    const UPDATE_THRESHOLD: f32 = 50.0; // Rebuild if King moves more than 50 units

    if let Ok(king_transform) = king_query.single() {
        let king_pos = Vec2::new(king_transform.translation.x, king_transform.translation.z);
        let distance_moved = king_pos.distance(pathfinding.last_king_pos);

        if distance_moved > UPDATE_THRESHOLD {
            // King moved significantly, queue attacker field rebuild
            pathfinding.pending_rebuilds.push(RebuildRequest {
                field_type: RebuildFieldType::Attacker,
                goal_pos: king_pos,
                dirty_cells: vec![], // Empty = full rebuild
            });
            pathfinding.last_king_pos = king_pos;
            debug!(
                "King moved {} units, rebuilding attacker field",
                distance_moved
            );
        }
    }
}
