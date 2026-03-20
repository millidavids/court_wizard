//! Pathfinding systems.

use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;

use crate::game::constants::{BATTLEFIELD_SIZE, defender_spawn_center};
use crate::game::units::archer::Archer;
use crate::game::units::assassin::Assassin;
use crate::game::units::assassin::constants as assassin_constants;
use crate::game::units::components::{Corpse, Team};
use crate::game::units::infantry::components::DefendersActivated;
use crate::game::units::infantry::Infantry;
use crate::game::units::king::components::King;

use crate::game::components::Acceleration;

use super::components::{FlowFieldInfluence, FlowFieldVelocity, StuckDetection};
use super::messages::{ObstacleChanged, ObstacleType};
use super::resources::PathfindingGrid;

/// Cell size for the pathfinding grid (in world units).
pub(crate) const CELL_SIZE: f32 = 10.0;

/// Buffer zone added around obstacles/hazards in the pathfinding grid.
/// Equal to one cell size so units start rerouting one cell before hitting the obstacle.
pub(crate) const OBSTACLE_BUFFER: f32 = CELL_SIZE;

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

/// Polls a pending flow field rebuild task. If complete, stores the result
/// and clears the pending task. Otherwise keeps it pending.
macro_rules! poll_rebuild {
    ($pending:expr, $field:expr) => {
        if let Some(mut task) = $pending.take() {
            if let Some(new_field) = bevy::tasks::block_on(bevy::tasks::poll_once(&mut task)) {
                $field = Some(new_field);
            } else {
                $pending = Some(task);
            }
        }
    };
}

/// Continuously rebuilds all active flow fields in parallel on background threads.
///
/// Each frame, polls pending async tasks and applies completed fields. When a field
/// has no pending rebuild, immediately spawns a new one with fresh target data.
/// This keeps all fields constantly up-to-date without timers or thresholds.
#[allow(clippy::type_complexity)]
pub fn continuous_flow_field_rebuild(
    mut pathfinding: ResMut<PathfindingGrid>,
    king_query: Query<(&Transform, &Team), With<King>>,
    all_transforms: Query<&Transform>,
    infantry_query: Query<(&Transform, &Team), (With<Infantry>, Without<Corpse>)>,
    archers: Query<(&Transform, &Team), (With<Archer>, Without<Corpse>)>,
    assassins: Query<(), (With<Assassin>, Without<Corpse>)>,
) {
    // --- Poll and apply completed rebuilds ---

    poll_rebuild!(pathfinding.pending_attacker_rebuild, pathfinding.attacker_field);
    poll_rebuild!(pathfinding.pending_defender_rebuild, pathfinding.defender_field);
    poll_rebuild!(pathfinding.pending_assassin_rebuild, pathfinding.assassin_field);

    // --- Spawn new rebuilds for any field not currently in flight ---

    // Find Defender King position (needed for attacker field)
    let king_pos = king_query
        .iter()
        .find(|(_, team)| **team == Team::Defenders)
        .map(|(t, _)| Vec2::new(t.translation.x, t.translation.z));

    // Attacker field: always rebuild toward King
    if pathfinding.pending_attacker_rebuild.is_none() {
        if let Some(pos) = king_pos {
            pathfinding.last_king_pos = pos;
            spawn_attacker_field_rebuild(&mut pathfinding, pos);
        }
    }

    // Defender field: rebuild toward current target or spawn center
    if pathfinding.pending_defender_rebuild.is_none() {
        if let Some(target_entity) = pathfinding.king_current_target
            && let Ok(target_transform) = all_transforms.get(target_entity)
        {
            let target_pos = Vec2::new(
                target_transform.translation.x,
                target_transform.translation.z,
            );
            pathfinding.last_defender_target_pos = target_pos;
            spawn_defender_field_rebuild(&mut pathfinding, target_pos, 0);
        } else if pathfinding.defender_field.is_some() {
            // No target — rally to spawn center
            let (cx, cz) = defender_spawn_center();
            spawn_defender_field_rebuild(
                &mut pathfinding,
                Vec2::new(cx, cz),
                DEFENDER_SPAWN_RALLY_RADIUS,
            );
        }
    }

    // Assassin field: rebuild toward archer center of mass (or King fallback)
    if pathfinding.pending_assassin_rebuild.is_none() {
        // Clear field when all assassins die
        if assassins.is_empty() {
            if pathfinding.assassin_field.is_some() {
                pathfinding.assassin_field = None;
            }
        } else {
            // Calculate center of mass of defender archers
            let mut archer_sum = Vec2::ZERO;
            let mut archer_count = 0u32;
            for (transform, team) in &archers {
                if *team == Team::Defenders {
                    archer_sum += Vec2::new(transform.translation.x, transform.translation.z);
                    archer_count += 1;
                }
            }

            let target_pos = if archer_count > 0 {
                archer_sum / archer_count as f32
            } else {
                // Fall back to King position
                king_pos.unwrap_or(Vec2::ZERO)
            };

            if target_pos != Vec2::ZERO {
                pathfinding.last_assassin_target_pos = target_pos;

                // Only avoid infantry when archers exist — when targeting King
                // directly, assassins should charge straight in
                let (attacker_positions, defender_positions) = if archer_count > 0 {
                    let mut atk = Vec::new();
                    let mut def = Vec::new();
                    for (t, team) in &infantry_query {
                        let pos = Vec2::new(t.translation.x, t.translation.z);
                        match *team {
                            Team::Attackers => atk.push(pos),
                            Team::Defenders => def.push(pos),
                            _ => atk.push(pos),
                        }
                    }
                    (atk, def)
                } else {
                    (Vec::new(), Vec::new())
                };

                spawn_assassin_field_rebuild(
                    &mut pathfinding,
                    target_pos,
                    &attacker_positions,
                    &defender_positions,
                );
            }
        }
    }
}

/// Tracks King's closest enemy target for the defender flow field.
///
/// Runs only when defenders are activated. The continuous rebuild system reads
/// `king_current_target` to determine where the defender field should point.
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
    let closest_enemy = enemy_query
        .iter()
        .filter(|(_, _, team)| **team != Team::Defenders)
        .map(|(entity, t, _)| {
            let d = king_pos.distance(Vec3::new(t.translation.x, 0.0, t.translation.z));
            (entity, d)
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    match closest_enemy {
        Some((new_entity, _)) => {
            pathfinding.king_current_target = Some(new_entity);
            pathfinding.defender_rally_delay = 0.0;
        }
        None => {
            // All enemies dead — start rally delay timer
            pathfinding.king_current_target = None;
            if pathfinding.defender_rally_delay <= 0.0 {
                pathfinding.defender_rally_delay = DEFENDER_RALLY_DELAY_SECS;
            }
        }
    }
}

/// Ticks the defender rally delay timer. When it expires, clears the defender
/// target so the continuous rebuild system builds toward spawn center.
pub fn tick_defender_rally_delay(mut pathfinding: ResMut<PathfindingGrid>, time: Res<Time>) {
    if pathfinding.defender_rally_delay <= 0.0 {
        return;
    }

    pathfinding.defender_rally_delay -= time.delta_secs();
    if pathfinding.defender_rally_delay <= 0.0 {
        pathfinding.defender_rally_delay = 0.0;
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

/// Spawns async task to rebuild the assassin flow field.
///
/// The assassin field routes toward archer center of mass while marking infantry
/// positions as high-cost terrain to encourage flanking. Attacker (friendly) infantry
/// gets a wider avoidance radius than defender (enemy) infantry.
fn spawn_assassin_field_rebuild(
    pathfinding: &mut PathfindingGrid,
    target_pos: Vec2,
    attacker_infantry_positions: &[Vec2],
    defender_infantry_positions: &[Vec2],
) {
    let task_pool = AsyncComputeTaskPool::get();

    let mut field = pathfinding.create_field_with_base_costs();
    let world_min = pathfinding.world_min;
    let cell_size = pathfinding.cell_size;
    let grid_width = pathfinding.grid_width;
    let grid_height = pathfinding.grid_height;
    let avoidance_cost = assassin_constants::INFANTRY_AVOIDANCE_COST;
    let attacker_radius = assassin_constants::ATTACKER_INFANTRY_AVOIDANCE_RADIUS as isize;
    let defender_radius = assassin_constants::DEFENDER_INFANTRY_AVOIDANCE_RADIUS as isize;

    let attacker_pos = attacker_infantry_positions.to_vec();
    let defender_pos = defender_infantry_positions.to_vec();

    let task = task_pool.spawn(async move {
        // Mark infantry positions as high-cost terrain with team-specific radii
        let mark_avoidance = |field: &mut super::flow_field::FlowField,
                              positions: &[Vec2],
                              radius: isize| {
            for pos in positions {
                let cx = ((pos.x - world_min.x) / cell_size).floor() as isize;
                let cz = ((pos.y - world_min.y) / cell_size).floor() as isize;

                for dz in -radius..=radius {
                    for dx in -radius..=radius {
                        let gx = cx + dx;
                        let gz = cz + dz;
                        if gx >= 0
                            && gz >= 0
                            && (gx as usize) < grid_width
                            && (gz as usize) < grid_height
                        {
                            let idx = gz as usize * grid_width + gx as usize;
                            if field.costs[idx] < avoidance_cost {
                                field.costs[idx] = avoidance_cost;
                            }
                        }
                    }
                }
            }
        };

        mark_avoidance(&mut field, &attacker_pos, attacker_radius);
        mark_avoidance(&mut field, &defender_pos, defender_radius);

        // Convert target position to grid coordinates
        let goal_x = ((target_pos.x - world_min.x) / cell_size).floor().max(0.0) as usize;
        let goal_z = ((target_pos.y - world_min.y) / cell_size).floor().max(0.0) as usize;

        field.generate(goal_x, goal_z, 0);
        field
    });

    pathfinding.pending_assassin_rebuild = Some(task);
}


/// Delay before rebuilding defender field toward spawn when all enemies die (seconds).
/// Prevents oscillation when enemies die rapidly between waves.
const DEFENDER_RALLY_DELAY_SECS: f32 = 2.0;

/// Handles obstacle change events — updates base_costs for walls, hazards, and terrain.
///
/// With continuous rebuilding, no explicit rebuild trigger is needed; the next rebuild
/// cycle will automatically pick up the updated base_costs.
pub fn handle_obstacle_events(
    mut obstacle_events: MessageReader<ObstacleChanged>,
    mut pathfinding: ResMut<PathfindingGrid>,
) {
    for event in obstacle_events.read() {
        let affected_cells = if let Some(shape) = &event.shape {
            pathfinding.shape_filtered_cells(event.bounds, shape)
        } else {
            pathfinding.world_bounds_to_cells(event.bounds)
        };

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

        let wmin = pathfinding.world_min;
        let cs = pathfinding.cell_size;

        match influence {
            FlowFieldInfluence::Attacker => {
                flow_velocity.at_destination = false;
                let (vel, dist) =
                    sample_field_or_zero(&pathfinding.attacker_field, world_pos, wmin, cs);
                flow_velocity.velocity = vel;
                flow_velocity.pathfinding_distance = dist;
            }
            FlowFieldInfluence::Defender { spawn_pos } => {
                if defenders_activated.active {
                    if let Some(ref field) = pathfinding.defender_field {
                        let direction = field.sample(world_pos, wmin, cs);
                        flow_velocity.velocity = direction;
                        flow_velocity.pathfinding_distance =
                            field.sample_distance(world_pos, wmin, cs);
                        flow_velocity.at_destination = direction == Vec3::ZERO;
                    } else {
                        rally_to_spawn(world_pos, spawn_pos, &mut flow_velocity);
                    }
                } else {
                    rally_to_spawn(world_pos, spawn_pos, &mut flow_velocity);
                }
            }
            FlowFieldInfluence::Assassin => {
                flow_velocity.at_destination = false;
                // Prefer assassin field, fall back to attacker field
                let field_ref = if pathfinding.assassin_field.is_some() {
                    &pathfinding.assassin_field
                } else {
                    &pathfinding.attacker_field
                };
                let (vel, dist) = sample_field_or_zero(field_ref, world_pos, wmin, cs);
                flow_velocity.velocity = vel;
                flow_velocity.pathfinding_distance = dist;
            }
        }
    }
}

/// Samples velocity and distance from a flow field, returning zeroes if the field is None.
fn sample_field_or_zero(
    field: &Option<super::flow_field::FlowField>,
    world_pos: Vec3,
    world_min: Vec2,
    cell_size: f32,
) -> (Vec3, f32) {
    if let Some(f) = field {
        (
            f.sample(world_pos, world_min, cell_size),
            f.sample_distance(world_pos, world_min, cell_size),
        )
    } else {
        (Vec3::ZERO, f32::INFINITY)
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
                let sign = if (transform.translation.x + transform.translation.z) as i32 % 2 == 0 {
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
