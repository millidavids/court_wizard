use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;

use crate::game::constants::defender_spawn_center;
use crate::game::units::archer::Archer;
use crate::game::units::assassin::Assassin;
use crate::game::units::assassin::constants as assassin_constants;
use crate::game::units::components::{Corpse, Team};
use crate::game::units::infantry::Infantry;
use crate::game::units::infantry::components::DefendersActivated;
use crate::game::units::king::components::King;

use crate::game::pathfinding::components::StagingAttacker;
use crate::game::pathfinding::resources::PathfindingGrid;

/// Satisfaction radius for defenders rallying to spawn points (in cells).
/// 200 units / 10 units per cell = 20 cells
pub(crate) const DEFENDER_SPAWN_RALLY_RADIUS: usize = 20;

/// Delay before rebuilding defender field toward spawn when all enemies die (seconds).
/// Prevents oscillation when enemies die rapidly between waves.
const DEFENDER_RALLY_DELAY_SECS: f32 = 2.0;

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
pub fn spawn_attacker_field_rebuild(pathfinding: &mut PathfindingGrid, king_pos: Vec2) {
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
pub fn spawn_defender_field_rebuild(
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
pub fn spawn_assassin_field_rebuild(
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
        let mark_avoidance = |field: &mut crate::game::pathfinding::flow_field::FlowField,
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
