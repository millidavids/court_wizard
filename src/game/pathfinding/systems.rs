//! Pathfinding systems.

use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;

use crate::game::constants::{BATTLEFIELD_SIZE, defender_spawn_center};
use crate::game::units::components::Team;
use crate::game::units::infantry::components::DefendersActivated;
use crate::game::units::king::components::King;

use crate::game::components::Acceleration;

use super::components::{FlowFieldInfluence, FlowFieldVelocity, StuckDetection};
use super::messages::{ObstacleChanged, ObstacleType};
use super::resources::{PathfindingGrid, RebuildTarget};

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
        pathfinding.last_king_pos = king_pos;
        pathfinding.enqueue_rebuild(RebuildTarget::Attacker);
        debug!(
            "King moved {} units, queuing attacker field rebuild",
            distance_moved
        );
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

    // Check if target changed — but only rebuild if the target POSITION moved
    // significantly, not just because a different entity became closest.
    match (pathfinding.king_current_target, closest_enemy) {
        (None, Some((new_entity, _))) => {
            // First time getting a target — rebuild toward enemy
            pathfinding.king_current_target = Some(new_entity);
            pathfinding.defender_rally_delay = 0.0;
            if let Ok((_, new_transform, _)) = enemy_query.get(new_entity) {
                pathfinding.last_defender_target_pos =
                    Vec2::new(new_transform.translation.x, new_transform.translation.z);
            }
            pathfinding.enqueue_rebuild(RebuildTarget::Defender);
            debug!("New defender target acquired, queuing defender field rebuild");
        }
        (Some(_old_entity), Some((new_entity, _))) => {
            // Update tracked entity (might be a different entity at a similar position)
            pathfinding.king_current_target = Some(new_entity);
            pathfinding.defender_rally_delay = 0.0;

            // Only rebuild if the new target position is significantly different
            if let Ok((_, new_transform, _)) = enemy_query.get(new_entity) {
                let new_pos =
                    Vec2::new(new_transform.translation.x, new_transform.translation.z);
                let distance = new_pos.distance(pathfinding.last_defender_target_pos);
                if distance > DEFENDER_TARGET_MOVEMENT_THRESHOLD {
                    pathfinding.last_defender_target_pos = new_pos;
                    if pathfinding.pending_defender_rebuild.is_none() {
                        pathfinding.enqueue_rebuild(RebuildTarget::Defender);
                        debug!(
                            "Defender target moved {} units, queuing rebuild",
                            distance
                        );
                    }
                }
            }
        }
        (Some(_), None) => {
            // All enemies dead — start rally delay timer instead of rebuilding immediately.
            // This prevents oscillation when enemies die and new ones spawn quickly.
            pathfinding.king_current_target = None;
            if pathfinding.defender_rally_delay <= 0.0 {
                pathfinding.defender_rally_delay = DEFENDER_RALLY_DELAY_SECS;
            }
        }
        _ => {}
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
///
/// `satisfaction_radius` is in grid cells — units within this radius of the goal
/// stop receiving flow directions. Use 0 for combat (charge in) or a larger
/// value for rally points so units spread out naturally.
fn spawn_defender_field_rebuild(
    pathfinding: &mut PathfindingGrid,
    target_pos: Vec2,
    satisfaction_radius: usize,
) {
    let task_pool = AsyncComputeTaskPool::get();

    let mut field = pathfinding.create_field_with_base_costs();
    let world_min = pathfinding.world_min;
    let cell_size = pathfinding.cell_size;

    let task = task_pool.spawn(async move {
        // Convert target position to grid coordinates
        let goal_x = ((target_pos.x - world_min.x) / cell_size).floor().max(0.0) as usize;
        let goal_z = ((target_pos.y - world_min.y) / cell_size).floor().max(0.0) as usize;

        field.generate(goal_x, goal_z, satisfaction_radius);
        field
    });

    pathfinding.pending_defender_rebuild = Some(task);
}

/// Polls pending async rebuild tasks and applies completed fields.
///
/// When `costs_dirty` is set (base_costs changed while a rebuild was in flight),
/// immediately triggers a fresh rebuild so the flow fields reflect all obstacles.
pub fn apply_completed_rebuilds(mut pathfinding: ResMut<PathfindingGrid>) {
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
    // applied are stale. Queue fresh rebuilds with the current base_costs.
    if pathfinding.costs_dirty && (attacker_done || defender_done) {
        pathfinding.costs_dirty = false;

        if attacker_done {
            pathfinding.enqueue_rebuild(RebuildTarget::Attacker);
        }
        if defender_done && pathfinding.defender_field.is_some() {
            pathfinding.enqueue_rebuild(RebuildTarget::Defender);
        }
    }
}

/// Debounce window for full rebuilds triggered by blocked obstacles (seconds).
const OBSTACLE_DEBOUNCE_SECS: f32 = 0.5;

/// Delay before rebuilding defender field toward spawn when all enemies die (seconds).
/// Prevents oscillation when enemies die rapidly between waves.
const DEFENDER_RALLY_DELAY_SECS: f32 = 2.0;

/// How far the defender's target must move before triggering a field rebuild (world units).
/// Prevents constant rebuilds when the closest enemy entity changes but the position
/// is effectively the same (e.g. enemies dying in the same cluster).
const DEFENDER_TARGET_MOVEMENT_THRESHOLD: f32 = 100.0;

/// Handles obstacle change events.
///
/// For **hazards/terrain/removed**: updates costs in base_costs AND applies a cheap
/// localized flow field update directly to both active fields. No full rebuild needed.
///
/// For **blocked** obstacles (walls): updates base_costs and starts a debounce timer
/// that triggers a full async rebuild, since walls fundamentally change pathing.
pub fn handle_obstacle_events(
    mut obstacle_events: MessageReader<ObstacleChanged>,
    mut pathfinding: ResMut<PathfindingGrid>,
) {
    let mut needs_full_rebuild = false;

    for event in obstacle_events.read() {
        // Narrowphase: filter cells by actual shape when provided.
        let affected_cells = if let Some(shape) = &event.shape {
            pathfinding.shape_filtered_cells(event.bounds, shape)
        } else {
            pathfinding.world_bounds_to_cells(event.bounds)
        };

        match event.obstacle_type {
            ObstacleType::Blocked => {
                pathfinding.mark_blocked(&affected_cells);
                needs_full_rebuild = true;
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
    }

    if needs_full_rebuild {
        pathfinding.rebuild_debounce = OBSTACLE_DEBOUNCE_SECS;
    }
}

/// Ticks the defender rally delay timer. When it expires (no new enemies appeared),
/// enqueues a defender field rebuild toward the spawn center.
pub fn tick_defender_rally_delay(
    mut pathfinding: ResMut<PathfindingGrid>,
    time: Res<Time>,
) {
    if pathfinding.defender_rally_delay <= 0.0 {
        return;
    }

    pathfinding.defender_rally_delay -= time.delta_secs();
    if pathfinding.defender_rally_delay > 0.0 {
        return;
    }
    pathfinding.defender_rally_delay = 0.0;

    // Timer expired — no new enemies appeared, rebuild toward spawn center
    pathfinding.enqueue_rebuild(RebuildTarget::Defender);
    debug!("Defender rally delay expired, queuing rebuild toward spawn center");
}

/// Ticks down the debounce timer and enqueues full rebuilds when it expires.
/// Only used for blocked obstacles (walls) that require a complete Dijkstra recalculation.
pub fn flush_debounced_rebuilds(mut pathfinding: ResMut<PathfindingGrid>, time: Res<Time>) {
    if pathfinding.rebuild_debounce <= 0.0 {
        return;
    }

    pathfinding.rebuild_debounce -= time.delta_secs();
    if pathfinding.rebuild_debounce > 0.0 {
        return;
    }
    pathfinding.rebuild_debounce = 0.0;

    pathfinding.enqueue_rebuild(RebuildTarget::Attacker);
    if pathfinding.defender_field.is_some() {
        pathfinding.enqueue_rebuild(RebuildTarget::Defender);
    }
}

/// Processes the rebuild queue, spawning at most one async rebuild per frame.
/// This spreads the cost of multiple rebuilds across frames to avoid spikes.
pub fn process_rebuild_queue(
    mut pathfinding: ResMut<PathfindingGrid>,
    king_query: Query<(&Transform, &Team), With<King>>,
    all_transforms: Query<&Transform>,
) {
    // Don't start a new rebuild if both slots are already occupied
    if pathfinding.pending_attacker_rebuild.is_some()
        && pathfinding.pending_defender_rebuild.is_some()
    {
        return;
    }

    // Find Defender King position (needed for attacker field target)
    let king_pos = king_query
        .iter()
        .find(|(_, team)| **team == Team::Defenders)
        .map(|(t, _)| Vec2::new(t.translation.x, t.translation.z));

    // Try to dequeue one rebuild that doesn't already have a pending task
    let queue_len = pathfinding.rebuild_queue.len();
    for _ in 0..queue_len {
        let Some(target) = pathfinding.rebuild_queue.pop_front() else {
            break;
        };

        match target {
            RebuildTarget::Attacker => {
                if pathfinding.pending_attacker_rebuild.is_some() {
                    // Already in flight — mark dirty so it re-queues on completion
                    pathfinding.costs_dirty = true;
                    continue;
                }
                if let Some(pos) = king_pos {
                    spawn_attacker_field_rebuild(&mut pathfinding, pos);
                    return; // One per frame
                }
            }
            RebuildTarget::Defender => {
                if pathfinding.pending_defender_rebuild.is_some() {
                    pathfinding.costs_dirty = true;
                    continue;
                }
                if let Some(target_entity) = pathfinding.king_current_target
                    && let Ok(target_transform) = all_transforms.get(target_entity)
                {
                    let target_pos = Vec2::new(
                        target_transform.translation.x,
                        target_transform.translation.z,
                    );
                    spawn_defender_field_rebuild(&mut pathfinding, target_pos, 0);
                } else if pathfinding.defender_field.is_some() {
                    // Only rally to spawn if a defender field was previously created
                    // (don't create a spawn-center field before defenders activate)
                    let (cx, cz) = defender_spawn_center();
                    spawn_defender_field_rebuild(
                        &mut pathfinding,
                        Vec2::new(cx, cz),
                        DEFENDER_SPAWN_RALLY_RADIUS,
                    );
                } else {
                    // No target and no existing field — skip (defenders not yet activated)
                    continue;
                }
                return; // One per frame
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
    mut units_query: Query<(&Transform, &FlowFieldInfluence, &mut FlowFieldVelocity)>,
) {
    for (transform, influence, mut flow_velocity) in units_query.iter_mut() {
        let world_pos = transform.translation;

        // Sample terrain cost from base costs (always up-to-date)
        flow_velocity.terrain_cost = pathfinding.sample_base_cost(world_pos);

        match influence {
            FlowFieldInfluence::Attacker => {
                // Sample attacker field
                flow_velocity.at_destination = false;
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

                        flow_velocity.velocity = direction;
                        flow_velocity.pathfinding_distance = field.sample_distance(
                            world_pos,
                            pathfinding.world_min,
                            pathfinding.cell_size,
                        );
                        // Mark at_destination when the flow field itself reports zero
                        // direction (unit is within the field's satisfaction radius)
                        flow_velocity.at_destination = direction == Vec3::ZERO;
                    } else {
                        // No defender field (no enemies between waves) — rally to spawn
                        rally_to_spawn(world_pos, spawn_pos, &mut flow_velocity);
                    }
                } else {
                    // Defenders not activated, rally to spawn point with satisfaction radius
                    rally_to_spawn(world_pos, spawn_pos, &mut flow_velocity);
                }
            }
        }
    }
}

/// Moves a defender toward its spawn position with a satisfaction radius.
fn rally_to_spawn(world_pos: Vec3, spawn_pos: &Vec2, flow_velocity: &mut FlowFieldVelocity) {
    let current_pos = Vec2::new(world_pos.x, world_pos.z);
    let distance_to_spawn = current_pos.distance(*spawn_pos);
    let satisfaction_radius_world = DEFENDER_SPAWN_RALLY_RADIUS as f32 * CELL_SIZE;

    if distance_to_spawn > satisfaction_radius_world {
        let direction = (*spawn_pos - current_pos).normalize();
        flow_velocity.velocity = Vec3::new(direction.x, 0.0, direction.y);
        flow_velocity.at_destination = false;
    } else {
        flow_velocity.velocity = Vec3::ZERO;
        flow_velocity.at_destination = true;
    }
    flow_velocity.pathfinding_distance = distance_to_spawn;
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

/// Auto-inserts `StuckDetection` on entities that have `FlowFieldVelocity` but
/// don't yet have `StuckDetection`. This avoids modifying every spawn function.
pub fn init_stuck_detection(
    mut commands: Commands,
    query: Query<(Entity, &Transform), (With<FlowFieldVelocity>, Without<StuckDetection>)>,
) {
    for (entity, transform) in &query {
        commands.entity(entity).insert(StuckDetection {
            last_check_pos: transform.translation,
            frames_since_check: 0,
            consecutive_stuck: 0,
        });
    }
}

/// Checks every 30 frames whether a unit has moved. After 3 consecutive stuck
/// checks (~1.5s), applies a perpendicular nudge force to break free.
pub fn detect_and_recover_stuck_units(
    mut query: Query<(
        &Transform,
        &FlowFieldVelocity,
        &mut StuckDetection,
        &mut Acceleration,
    )>,
) {
    const CHECK_INTERVAL: u32 = 30;
    const STUCK_THRESHOLD: f32 = 2.0;
    const STUCK_COUNT_FOR_NUDGE: u32 = 3;
    const NUDGE_FORCE: f32 = 400.0;

    for (transform, flow_vel, mut stuck, mut accel) in &mut query {
        stuck.frames_since_check += 1;
        if stuck.frames_since_check < CHECK_INTERVAL {
            continue;
        }
        stuck.frames_since_check = 0;

        // Skip units at destination or with no flow velocity
        if flow_vel.at_destination || flow_vel.velocity.length_squared() < 0.01 {
            stuck.consecutive_stuck = 0;
            stuck.last_check_pos = transform.translation;
            continue;
        }

        let distance_moved = transform.translation.distance(stuck.last_check_pos);

        if distance_moved < STUCK_THRESHOLD {
            stuck.consecutive_stuck += 1;

            if stuck.consecutive_stuck >= STUCK_COUNT_FOR_NUDGE {
                // Apply perpendicular nudge to flow velocity direction.
                // Alternate direction based on position hash for variety.
                let flow_dir = flow_vel.velocity.normalize_or_zero();
                let perp = Vec3::new(-flow_dir.z, 0.0, flow_dir.x);

                // Use position to determine nudge direction consistently
                let sign = if (transform.translation.x + transform.translation.z) as i32 % 2 == 0
                {
                    1.0
                } else {
                    -1.0
                };

                accel.add_force(perp * NUDGE_FORCE * sign);
                stuck.consecutive_stuck = 0;
            }
        } else {
            stuck.consecutive_stuck = 0;
        }

        stuck.last_check_pos = transform.translation;
    }
}
