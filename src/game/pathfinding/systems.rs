//! Pathfinding systems.

use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;

use crate::game::constants::BATTLEFIELD_SIZE;
use crate::game::units::archer::Archer;
use crate::game::units::components::Team;
use crate::game::units::infantry::components::DefendersActivated;
use crate::game::units::king::components::King;

use super::components::{FlowFieldInfluence, FlowFieldVelocity};
use super::messages::{ObstacleChanged, ObstacleType};
use super::resources::PathfindingGrid;

/// Cell size for the pathfinding grid (in world units).
pub(crate) const CELL_SIZE: f32 = 10.0;

/// Buffer zone added around obstacles/hazards in the pathfinding grid.
/// Equal to one cell size so units start rerouting one cell before hitting the obstacle.
pub(crate) const OBSTACLE_BUFFER: f32 = CELL_SIZE;

/// Distance threshold for King movement to trigger attacker field rebuild.
/// Set just below the 200-unit targeting crossover so units arriving at the
/// old goal position will already be in targeting-dominant range.
const KING_MOVEMENT_THRESHOLD: f32 = 180.0;

/// Satisfaction radius for defenders rallying to spawn points (in cells).
/// 200 units / 10 units per cell = 20 cells
const DEFENDER_SPAWN_RALLY_RADIUS: usize = 20;

/// Initializes the pathfinding grid resource.
pub fn initialize_pathfinding(mut commands: Commands) {
    let pathfinding = PathfindingGrid::new(BATTLEFIELD_SIZE, CELL_SIZE);

    info!(
        "Pathfinding initialized: {}x{} grid ({} cells, {} bytes per field)",
        pathfinding.grid_width,
        pathfinding.grid_height,
        pathfinding.grid_width * pathfinding.grid_height,
        pathfinding.grid_width * pathfinding.grid_height * std::mem::size_of::<Vec3>()
    );

    commands.insert_resource(pathfinding);
}

/// Updates King position and spawns attacker field rebuild when King moves significantly.
///
/// Tracks the **Defender King** specifically, since the attacker flow field guides
/// guest-side units toward the host's (Defender) King. In single-player there is
/// only one King (always Defenders), so the filter is a no-op.
pub fn update_king_position(
    mut pathfinding: ResMut<PathfindingGrid>,
    king_query: Query<(&Transform, &Team), With<King>>,
) {
    let Some((king_transform, _)) = king_query
        .iter()
        .find(|(_, team)| **team == Team::Defenders)
    else {
        return;
    };

    let king_pos = Vec2::new(king_transform.translation.x, king_transform.translation.z);
    let distance_moved = king_pos.distance(pathfinding.last_king_pos);

    if distance_moved > KING_MOVEMENT_THRESHOLD {
        // King moved significantly, spawn attacker field rebuild
        if pathfinding.pending_attacker_rebuild.is_none() {
            spawn_attacker_field_rebuild(&mut pathfinding, king_pos);
            pathfinding.last_king_pos = king_pos;
            debug!(
                "King moved {} units, rebuilding attacker field",
                distance_moved
            );
        }
    }
}

/// Selects King's target and spawns defender field rebuild when needed.
///
/// Runs only when defenders are activated.
pub fn update_king_target(
    mut pathfinding: ResMut<PathfindingGrid>,
    defenders_activated: Res<DefendersActivated>,
    king_query: Query<(&Transform, &Team), With<King>>,
    enemy_query: Query<
        (Entity, &Transform, &Team),
        (
            Without<King>,
            Without<crate::game::units::components::Corpse>,
        ),
    >,
) {
    // Only run when defenders are activated
    if !defenders_activated.active {
        return;
    }

    let Some((king_transform, _)) = king_query
        .iter()
        .find(|(_, team)| **team == Team::Defenders)
    else {
        return;
    };

    let king_pos = Vec3::new(
        king_transform.translation.x,
        0.0,
        king_transform.translation.z,
    );

    // Find closest enemy to King (only Attackers and Undead, not Defenders)
    let mut closest_enemy: Option<(Entity, f32)> = None;

    for (entity, enemy_transform, enemy_team) in enemy_query.iter() {
        // Skip defenders - only target Attackers and Undead
        if *enemy_team == Team::Defenders {
            continue;
        }

        let enemy_pos = Vec3::new(
            enemy_transform.translation.x,
            0.0,
            enemy_transform.translation.z,
        );
        let distance = king_pos.distance(enemy_pos);

        if let Some((_, current_closest)) = closest_enemy {
            if distance < current_closest {
                closest_enemy = Some((entity, distance));
            }
        } else {
            closest_enemy = Some((entity, distance));
        }
    }

    // Check if target changed
    let target_changed = match (pathfinding.king_current_target, closest_enemy) {
        (None, Some((new_entity, _))) => {
            // First time activating
            pathfinding.king_current_target = Some(new_entity);
            true
        }
        (Some(old_entity), Some((new_entity, _))) if old_entity != new_entity => {
            // Target changed
            pathfinding.king_current_target = Some(new_entity);
            true
        }
        (Some(_), None) => {
            // Old target died, no new targets (victory condition elsewhere)
            pathfinding.king_current_target = None;
            pathfinding.defender_field = None;
            false
        }
        _ => false,
    };

    // Spawn defender field rebuild if target changed and no rebuild pending
    if target_changed
        && pathfinding.pending_defender_rebuild.is_none()
        && let Some((target_entity, _)) = closest_enemy
    {
        // Get target position
        if let Ok((_, target_transform, _)) = enemy_query.get(target_entity) {
            let target_pos = Vec2::new(
                target_transform.translation.x,
                target_transform.translation.z,
            );
            spawn_defender_field_rebuild(&mut pathfinding, target_pos);
            debug!("King target changed, rebuilding defender field");
        }
    }
}

/// Spawns async task to rebuild the attacker flow field.
fn spawn_attacker_field_rebuild(pathfinding: &mut PathfindingGrid, king_pos: Vec2) {
    let task_pool = AsyncComputeTaskPool::get();

    let mut field = pathfinding.create_field_with_base_costs();
    let world_min = pathfinding.world_min;
    let cell_size = pathfinding.cell_size;

    let task = task_pool.spawn(async move {
        // Convert king position to grid coordinates
        let goal_x = ((king_pos.x - world_min.x) / cell_size).floor().max(0.0) as usize;
        let goal_z = ((king_pos.y - world_min.y) / cell_size).floor().max(0.0) as usize;

        // Generate field with 0 satisfaction radius
        field.generate(goal_x, goal_z, 0);
        field
    });

    pathfinding.pending_attacker_rebuild = Some(task);
}

/// Spawns async task to rebuild the defender flow field.
fn spawn_defender_field_rebuild(pathfinding: &mut PathfindingGrid, target_pos: Vec2) {
    let task_pool = AsyncComputeTaskPool::get();

    let mut field = pathfinding.create_field_with_base_costs();
    let world_min = pathfinding.world_min;
    let cell_size = pathfinding.cell_size;

    let task = task_pool.spawn(async move {
        // Convert target position to grid coordinates
        let goal_x = ((target_pos.x - world_min.x) / cell_size).floor().max(0.0) as usize;
        let goal_z = ((target_pos.y - world_min.y) / cell_size).floor().max(0.0) as usize;

        // Generate field with 0 satisfaction radius (infantry charges in)
        // Archers will handle their own satisfaction via targeting system
        field.generate(goal_x, goal_z, 0);
        field
    });

    pathfinding.pending_defender_rebuild = Some(task);
}

/// Polls pending async rebuild tasks and applies completed fields.
///
/// When `costs_dirty` is set (base_costs changed while a rebuild was in flight),
/// immediately triggers a fresh rebuild so the flow fields reflect all obstacles.
pub fn apply_completed_rebuilds(
    mut pathfinding: ResMut<PathfindingGrid>,
    king_query: Query<(&Transform, &Team), With<King>>,
    all_transforms: Query<&Transform>,
) {
    let mut attacker_done = false;
    let mut defender_done = false;

    // Check attacker field rebuild
    if let Some(mut task) = pathfinding.pending_attacker_rebuild.take() {
        if let Some(new_field) = bevy::tasks::block_on(bevy::tasks::poll_once(&mut task)) {
            pathfinding.attacker_field = Some(new_field);
            attacker_done = true;
        } else {
            pathfinding.pending_attacker_rebuild = Some(task);
        }
    }

    // Check defender field rebuild
    if let Some(mut task) = pathfinding.pending_defender_rebuild.take() {
        if let Some(new_field) = bevy::tasks::block_on(bevy::tasks::poll_once(&mut task)) {
            pathfinding.defender_field = Some(new_field);
            defender_done = true;
        } else {
            pathfinding.pending_defender_rebuild = Some(task);
        }
    }

    // If base_costs changed while rebuilds were in flight, the fields we just
    // applied are stale.  Kick off fresh rebuilds with the current base_costs.
    if pathfinding.costs_dirty && (attacker_done || defender_done) {
        pathfinding.costs_dirty = false;

        if attacker_done
            && let Some((king_transform, _)) = king_query
                .iter()
                .find(|(_, team)| **team == Team::Defenders)
        {
            let king_pos =
                Vec2::new(king_transform.translation.x, king_transform.translation.z);
            spawn_attacker_field_rebuild(&mut pathfinding, king_pos);
        }

        if defender_done
            && pathfinding.defender_field.is_some()
            && let Some(target_entity) = pathfinding.king_current_target
            && let Ok(target_transform) = all_transforms.get(target_entity)
        {
            let target_pos =
                Vec2::new(target_transform.translation.x, target_transform.translation.z);
            spawn_defender_field_rebuild(&mut pathfinding, target_pos);
        }
    }
}

/// Handles obstacle change events and updates base costs, then triggers rebuilds.
pub fn handle_obstacle_events(
    mut obstacle_events: MessageReader<ObstacleChanged>,
    mut pathfinding: ResMut<PathfindingGrid>,
    king_query: Query<(&Transform, &Team), With<King>>,
    all_transforms: Query<&Transform>,
) {
    let mut rebuild_needed = false;

    for event in obstacle_events.read() {
        // Convert world bounds to grid cells
        let affected_cells = pathfinding.world_bounds_to_cells(event.bounds);

        match event.obstacle_type {
            ObstacleType::Blocked => {
                pathfinding.mark_blocked(&affected_cells);
            }
            ObstacleType::SlowTerrain(multiplier) => {
                pathfinding.set_terrain_cost(&affected_cells, multiplier);
            }
            ObstacleType::Hazard(cost) => {
                pathfinding.set_terrain_cost(&affected_cells, cost);
            }
            ObstacleType::Removed => {
                pathfinding.set_terrain_cost(&affected_cells, 1.0);
            }
        }

        rebuild_needed = true;
    }

    // If any obstacles changed, rebuild both fields
    if rebuild_needed {
        // Rebuild attacker field toward Defender King
        if let Some((king_transform, _)) = king_query
            .iter()
            .find(|(_, team)| **team == Team::Defenders)
        {
            if pathfinding.pending_attacker_rebuild.is_none() {
                let king_pos =
                    Vec2::new(king_transform.translation.x, king_transform.translation.z);
                spawn_attacker_field_rebuild(&mut pathfinding, king_pos);
            } else {
                // A rebuild is already running from an older snapshot of base_costs.
                // Mark dirty so we re-rebuild once it completes.
                pathfinding.costs_dirty = true;
            }
        }

        // Rebuild defender field toward King's current target
        if pathfinding.defender_field.is_some()
            && let Some(target_entity) = pathfinding.king_current_target
            && let Ok(target_transform) = all_transforms.get(target_entity)
        {
            if pathfinding.pending_defender_rebuild.is_none() {
                let target_pos =
                    Vec2::new(target_transform.translation.x, target_transform.translation.z);
                spawn_defender_field_rebuild(&mut pathfinding, target_pos);
            } else {
                pathfinding.costs_dirty = true;
            }
        }
    }
}

/// Samples flow fields and updates FlowFieldVelocity for all units.
///
/// Runs before movement systems to provide flow field guidance.
pub fn sample_flow_fields(
    pathfinding: Res<PathfindingGrid>,
    defenders_activated: Res<DefendersActivated>,
    mut units_query: Query<(
        &Transform,
        &FlowFieldInfluence,
        &mut FlowFieldVelocity,
        Option<&Archer>,
    )>,
) {
    for (transform, influence, mut flow_velocity, is_archer) in units_query.iter_mut() {
        let world_pos = transform.translation;

        // Sample terrain cost from base costs (always up-to-date)
        flow_velocity.terrain_cost = pathfinding.sample_base_cost(world_pos);

        match influence {
            FlowFieldInfluence::Attacker => {
                // Sample attacker field
                if let Some(ref field) = pathfinding.attacker_field {
                    flow_velocity.velocity =
                        field.sample(world_pos, pathfinding.world_min, pathfinding.cell_size);
                    flow_velocity.pathfinding_distance = field.sample_distance(
                        world_pos,
                        pathfinding.world_min,
                        pathfinding.cell_size,
                    );
                } else {
                    flow_velocity.velocity = Vec3::ZERO;
                    flow_velocity.pathfinding_distance = f32::INFINITY;
                }
            }
            FlowFieldInfluence::Defender { spawn_pos } => {
                if defenders_activated.active {
                    // Defenders are activated, use defender flow field
                    if let Some(ref field) = pathfinding.defender_field {
                        let direction =
                            field.sample(world_pos, pathfinding.world_min, pathfinding.cell_size);

                        // Archers should respect their attack range as satisfaction radius
                        // If the flow field says "go forward" but they're already in range,
                        // zero out the flow field velocity so targeting takes over
                        if is_archer.is_some() && direction.length() > 0.0 {
                            // Check if archer is within attack range of target
                            if let Some(_target) = pathfinding.king_current_target {
                                // We can't easily get target position here without another query
                                // So we'll let the targeting system handle this via weighting
                                // The archer targeting system already handles optimal range positioning
                            }
                        }

                        flow_velocity.velocity = direction;
                        flow_velocity.pathfinding_distance = field.sample_distance(
                            world_pos,
                            pathfinding.world_min,
                            pathfinding.cell_size,
                        );
                    } else {
                        flow_velocity.velocity = Vec3::ZERO;
                        flow_velocity.pathfinding_distance = f32::INFINITY;
                    }
                } else {
                    // Defenders not activated, rally to spawn point with satisfaction radius
                    let current_pos = Vec2::new(world_pos.x, world_pos.z);
                    let distance_to_spawn = current_pos.distance(*spawn_pos);
                    let satisfaction_radius_world = DEFENDER_SPAWN_RALLY_RADIUS as f32 * CELL_SIZE;

                    if distance_to_spawn > satisfaction_radius_world {
                        // Move toward spawn point
                        let direction = (*spawn_pos - current_pos).normalize();
                        flow_velocity.velocity = Vec3::new(direction.x, 0.0, direction.y);
                    } else {
                        // Within satisfaction radius of spawn point
                        flow_velocity.velocity = Vec3::ZERO;
                    }
                    // Use straight-line distance when rallying to spawn
                    flow_velocity.pathfinding_distance = distance_to_spawn;
                }
            }
        }
    }
}

/// Generates initial attacker flow field on first frame after Defender King spawns.
///
/// This ensures the attacker field exists when attackers spawn.
pub fn generate_initial_fields(
    mut pathfinding: ResMut<PathfindingGrid>,
    king_query: Query<(&Transform, &Team), (With<King>, Added<Transform>)>,
) {
    // Only run when the Defender King first spawns
    let Some((king_transform, _)) = king_query
        .iter()
        .find(|(_, team)| **team == Team::Defenders)
    else {
        return;
    };

    let king_pos = Vec2::new(king_transform.translation.x, king_transform.translation.z);
    pathfinding.last_king_pos = king_pos;

    // Spawn initial attacker field rebuild
    spawn_attacker_field_rebuild(&mut pathfinding, king_pos);
    info!("Generating initial attacker flow field toward Defender King");
}
