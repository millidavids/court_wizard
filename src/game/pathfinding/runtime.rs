//! Pathfinding runtime: flow-field rebuild, sampling, and obstacle response.

use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;

use crate::game::constants::{CENTER_STAGING_INDEX, defender_spawn_center};
use crate::game::units::archer::Archer;
use crate::game::units::assassin::Assassin;
use crate::game::units::assassin::constants as assassin_constants;
use crate::game::units::components::{Corpse, Team};
use crate::game::units::infantry::Infantry;
use crate::game::units::infantry::components::DefendersActivated;
use crate::game::units::king::components::King;

use super::components::{FlowFieldInfluence, FlowFieldVelocity, StagingAttacker, WaveGroup};
use super::messages::{ObstacleChanged, ObstacleType};
use super::resources::PathfindingGrid;

use super::setup::CELL_SIZE;

/// Satisfaction radius for defenders rallying to spawn points (in cells).
/// 200 units / 10 units per cell = 20 cells
const DEFENDER_SPAWN_RALLY_RADIUS: usize = 20;

/// Initializes the pathfinding grid resource and registers static terrain obstacles
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

pub fn continuous_flow_field_rebuild(
    mut pathfinding: ResMut<PathfindingGrid>,
    king_query: Query<(&Transform, &Team), With<King>>,
    all_transforms: Query<&Transform>,
    infantry_query: Query<(&Transform, &Team), (With<Infantry>, Without<Corpse>)>,
    archers: Query<(&Transform, &Team), (With<Archer>, Without<Corpse>)>,
    assassins: Query<(), (With<Assassin>, Without<Corpse>)>,
) {
    // --- Poll and apply completed rebuilds ---

    poll_rebuild!(
        pathfinding.pending_attacker_rebuild,
        pathfinding.attacker_field
    );
    poll_rebuild!(
        pathfinding.pending_defender_rebuild,
        pathfinding.defender_field
    );
    poll_rebuild!(
        pathfinding.pending_assassin_rebuild,
        pathfinding.assassin_field
    );

    // --- Spawn new rebuilds for any field not currently in flight ---

    // Find Defender King position (needed for attacker field)
    let king_pos = king_query
        .iter()
        .find(|(_, team)| **team == Team::Defenders)
        .map(|(t, _)| Vec2::new(t.translation.x, t.translation.z));

    // Attacker field: always rebuild toward King
    if pathfinding.pending_attacker_rebuild.is_none()
        && let Some(pos) = king_pos
    {
        pathfinding.last_king_pos = pos;
        spawn_attacker_field_rebuild(&mut pathfinding, pos);
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
            Without<StagingAttacker>,
        ),
    >,
) {
    if !defenders_activated.active {
        pathfinding.king_current_target = None;
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
pub(super) fn spawn_attacker_field_rebuild(pathfinding: &mut PathfindingGrid, king_pos: Vec2) {
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
pub(super) fn spawn_defender_field_rebuild(
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
pub(super) fn spawn_assassin_field_rebuild(
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
        let mark_avoidance =
            |field: &mut super::flow_field::FlowField, positions: &[Vec2], radius: isize| {
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
    mut units_query: Query<(
        &Transform,
        &FlowFieldInfluence,
        &mut FlowFieldVelocity,
        Option<&StagingAttacker>,
        Has<WaveGroup>,
        &Team,
    )>,
) {
    for (transform, influence, mut flow_velocity, staging, has_wave_group, team) in
        units_query.iter_mut()
    {
        let world_pos = transform.translation;

        // Sample terrain cost from base costs (always up-to-date)
        flow_velocity.terrain_cost = pathfinding.sample_base_cost(world_pos);

        let wmin = pathfinding.world_min;
        let cs = pathfinding.cell_size;

        // Attackers use the staging field until activated:
        // - has StagingAttacker component, OR
        // - doesn't have WaveGroup yet (1-frame command delay after spawn)
        let is_staging = crate::game::units::systems::is_staging_attacker(
            team,
            staging.is_some(),
            has_wave_group,
        );

        match influence {
            FlowFieldInfluence::Attacker | FlowFieldInfluence::Assassin if is_staging => {
                // Staging: follow the staging field for this unit's assigned staging point
                let field = staging
                    .map(|s| &pathfinding.staging_fields[s.0 as usize])
                    .unwrap_or(&pathfinding.staging_fields[CENTER_STAGING_INDEX]);
                let (vel, dist) = sample_field_or_zero(field, world_pos, wmin, cs);
                flow_velocity.velocity = vel;
                flow_velocity.pathfinding_distance = dist;
                flow_velocity.at_destination = vel == Vec3::ZERO;
            }
            FlowFieldInfluence::Attacker => {
                // Activated: follow attacker field toward King
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
                // Activated: follow assassin field, fall back to attacker field
                flow_velocity.at_destination = false;
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
