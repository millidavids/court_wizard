//! Pathfinding setup: grid initialization and static terrain registration.

use bevy::prelude::*;

use crate::game::constants::{
    BATTLEFIELD_SIZE, CENTER_STAGING_INDEX, PATHFINDING_X_EXTENSION, STAGING_ACTIVATION_RADIUS,
    STAGING_POINT_COUNT, STAGING_POINTS, STAGING_SATISFACTION_RADIUS, WAVE_ACTIVATION_THRESHOLD,
    WAVE_STAGING_TIMEOUT,
};
use crate::game::units::components::{Corpse, Team};
use crate::game::units::king::components::King;

use crate::game::components::Acceleration;

use super::components::{
    FlowFieldInfluence, FlowFieldVelocity, StagingAttacker, StuckDetection, WaveGroup,
};
use super::resources::PathfindingGrid;
use super::runtime::spawn_attacker_field_rebuild;

/// Cell size for the pathfinding grid (in world units).
pub(crate) const CELL_SIZE: f32 = 10.0;

/// Buffer zone added around obstacles/hazards in the pathfinding grid.
/// Equal to one cell size so units start rerouting one cell before hitting the obstacle.
pub(crate) const OBSTACLE_BUFFER: f32 = CELL_SIZE;

/// Initializes the pathfinding grid resource and registers static terrain obstacles
pub fn initialize_pathfinding(mut commands: Commands) {
    let mut pathfinding =
        PathfindingGrid::new(BATTLEFIELD_SIZE, CELL_SIZE, PATHFINDING_X_EXTENSION);

    info!(
        "Pathfinding initialized: {}x{} grid ({} cells, {} bytes per field)",
        pathfinding.grid_width,
        pathfinding.grid_height,
        pathfinding.grid_width * pathfinding.grid_height,
        pathfinding.grid_width * pathfinding.grid_height * std::mem::size_of::<Vec3>()
    );

    // Register static terrain on the base costs directly (no need for messages
    // since the grid isn't inserted yet — messages would be processed next frame).
    register_static_terrain(&mut pathfinding);

    // Build all 7 staging flow fields once (never change — each targets one staging point).
    build_staging_fields(&mut pathfinding);

    commands.insert_resource(pathfinding);
}

/// Builds static staging flow fields for all staging points.
/// Each field guides unactivated attackers from the spawn area to one staging point.
fn build_staging_fields(pathfinding: &mut PathfindingGrid) {
    let world_min = pathfinding.world_min;
    let cell_size = pathfinding.cell_size;

    for (i, &(sx, sz)) in STAGING_POINTS.iter().enumerate() {
        let mut field = pathfinding.create_field_with_base_costs();

        let goal_x = ((sx - world_min.x) / cell_size).floor().max(0.0) as usize;
        let goal_z = ((sz - world_min.y) / cell_size).floor().max(0.0) as usize;

        field.generate(goal_x, goal_z, STAGING_SATISFACTION_RADIUS);
        pathfinding.staging_fields[i] = Some(field);
    }
    info!(
        "Built {} staging flow fields, satisfaction_radius={}",
        STAGING_POINT_COUNT, STAGING_SATISFACTION_RADIUS
    );
}

/// Registers lava hazard and water slow terrain directly on the pathfinding
/// grid's base costs. The right wall is purely visual (no flow field collision).
fn register_static_terrain(pathfinding: &mut PathfindingGrid) {
    use crate::game::battlefield::constants::{
        LAVA_AVOIDANCE_RADIUS, LAVA_HAZARD_FLOW_COST, LAVA_POOL_POSITION, WATER_POOL_POSITION,
        WATER_POOL_RADIUS, WATER_SLOW_FLOW_COST,
    };

    register_circular_terrain(
        pathfinding,
        LAVA_POOL_POSITION,
        LAVA_AVOIDANCE_RADIUS,
        LAVA_HAZARD_FLOW_COST,
        "Lava hazard",
    );
    register_circular_terrain(
        pathfinding,
        WATER_POOL_POSITION,
        WATER_POOL_RADIUS,
        WATER_SLOW_FLOW_COST,
        "Water slow terrain",
    );
}

/// Registers a circular terrain hazard on the pathfinding grid's base costs.
fn register_circular_terrain(
    pathfinding: &mut PathfindingGrid,
    position: Vec3,
    radius: f32,
    cost: f32,
    label: &str,
) {
    let center = Vec2::new(position.x, position.z);
    let shape = super::messages::ObstacleShape::circle(center, radius);
    let bounds = Rect::new(
        center.x - radius,
        center.y - radius,
        center.x + radius,
        center.y + radius,
    );
    let cells = pathfinding.shape_filtered_cells(bounds, &shape);
    pathfinding.set_terrain_cost(&cells, cost);
    info!("{}: {} cells at cost {}", label, cells.len(), cost);
}

/// Continuously rebuilds all active flow fields in parallel on background threads.
///
/// Each frame, polls pending async tasks and applies completed fields. When a field
/// has no pending rebuild, immediately spawns a new one with fresh target data.
/// This keeps all fields constantly up-to-date without timers or thresholds.
#[allow(clippy::type_complexity)]
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

/// Auto-tags newly spawned attackers with `StagingAttacker` and `WaveGroup`.
/// Detects entities with `FlowFieldInfluence::Attacker` that don't yet have a `WaveGroup`.
/// Assigns staging points from the `WaveStagingPlan` via round-robin.
/// Bosses always go to the center staging point (index 3).
/// Lazily computes the staging plan for a wave the first time attackers appear for it.
#[allow(clippy::too_many_arguments)]
pub fn tag_new_attackers(
    mut commands: Commands,
    wave_state: Option<Res<crate::game::resources::WaveState>>,
    game_seed: Option<Res<crate::game::seeded_rng::resources::GameSeed>>,
    current_level: Res<crate::game::resources::CurrentLevel>,
    mut staging_plan: ResMut<super::staging::WaveStagingPlan>,
    new_attackers: Query<
        (
            Entity,
            &Transform,
            &Team,
            &FlowFieldInfluence,
            Has<crate::game::units::boss::components::Boss>,
        ),
        (Without<WaveGroup>, Without<Corpse>),
    >,
) {
    let wave = wave_state.map(|w| w.current_wave).unwrap_or(0);

    if !staging_plan.has_wave(wave) {
        let seed = game_seed.as_ref().map(|s| s.0).unwrap_or(0);
        super::staging::compute_wave_staging(&mut staging_plan, seed, current_level.0, wave);
    }

    // Tunnel 0 (upper, z ≈ -375) → Left, tunnel 1 (lower, z ≈ -1575) → Right.
    let tunnel_z_midpoint = (crate::game::constants::ATTACKER_SPAWN_POINTS[0].1
        + crate::game::constants::ATTACKER_SPAWN_POINTS[1].1)
        / 2.0;

    for (entity, transform, team, influence, is_boss) in &new_attackers {
        if *team != Team::Attackers {
            continue;
        }
        match influence {
            FlowFieldInfluence::Attacker | FlowFieldInfluence::Assassin => {
                let staging_idx = if is_boss {
                    CENTER_STAGING_INDEX as u8
                } else {
                    let tunnel = if transform.translation.z > tunnel_z_midpoint {
                        super::staging::SpawnTunnel::Left
                    } else {
                        super::staging::SpawnTunnel::Right
                    };
                    staging_plan.next_staging_point(wave, tunnel)
                };
                commands
                    .entity(entity)
                    .insert((StagingAttacker(staging_idx), WaveGroup(wave)));
            }
            _ => {}
        }
    }
}

/// Tracks how long each wave has been in staging, for timeout-based force activation.
#[derive(Resource, Default)]
pub struct WaveStagingTimers {
    timers: std::collections::HashMap<u32, f32>,
}

/// Checks if 90% of a wave's living staging attackers have reached their
/// assigned staging points. Each unit is checked against its own staging point.
/// Also force-activates after a timeout to prevent stalling.
/// Dead units (Corpse) are excluded so lava kills don't block activation.
pub fn check_wave_activation(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut kill_stats: ResMut<crate::game::resources::KillStats>,
    mut staging_timers: ResMut<WaveStagingTimers>,
    mut staging_plan: ResMut<super::staging::WaveStagingPlan>,
    staging_query: Query<(Entity, &WaveGroup, &StagingAttacker, &Transform), Without<Corpse>>,
) {
    use std::collections::HashMap;

    let activation_radius_sq = STAGING_ACTIVATION_RADIUS * STAGING_ACTIVATION_RADIUS;
    // Use real time so 3x speedup doesn't shorten the timeout
    let dt = time.delta_secs();

    // Count total and arrived (within activation radius of their assigned point) per wave
    let mut wave_counts: HashMap<u32, (u32, u32)> = HashMap::new();
    for (_entity, wave_group, staging, transform) in &staging_query {
        let (total, arrived) = wave_counts.entry(wave_group.0).or_insert((0, 0));
        *total += 1;
        let point = STAGING_POINTS[staging.0 as usize];
        let staging_pos = Vec2::new(point.0, point.1);
        let unit_pos = Vec2::new(transform.translation.x, transform.translation.z);
        if unit_pos.distance_squared(staging_pos) <= activation_radius_sq {
            *arrived += 1;
        }
    }

    // Check each wave
    for (wave, (total, arrived)) in &wave_counts {
        if *total == 0 {
            continue;
        }

        // Tick staging timer for this wave
        let elapsed = staging_timers.timers.entry(*wave).or_insert(0.0);
        *elapsed += dt;

        let ratio = *arrived as f32 / *total as f32;
        let timed_out = *elapsed >= WAVE_STAGING_TIMEOUT;

        if ratio >= WAVE_ACTIVATION_THRESHOLD || timed_out {
            if timed_out {
                info!(
                    "Wave {} force-activated after {:.1}s timeout ({}/{} units within radius)",
                    wave, elapsed, arrived, total
                );
            } else {
                info!(
                    "Wave {} activated! ({}/{} units within staging radius)",
                    wave, arrived, total
                );
            }
            // Mark battle as started (enables the game timer)
            if !kill_stats.battle_started {
                kill_stats.battle_started = true;
                info!("Battle started — game timer running");
            }
            // Remove StagingAttacker from all units of this wave
            for (entity, wave_group, _, _) in &staging_query {
                if wave_group.0 == *wave {
                    commands.entity(entity).remove::<StagingAttacker>();
                }
            }
            staging_timers.timers.remove(wave);
            staging_plan.remove_wave(*wave);
        }
    }
}

/// Manages game speed: 3x when only staging (unactivated) attackers exist, 1x otherwise.
///
/// Drops the speedup to baseline whenever a full-screen in-game menu (spell
/// book / cauldron) is open — menu navigation and scrolling behave erratically
/// at 3x. When the player returns to `Running` with staging still active the
/// speedup resumes automatically on the next frame.
#[allow(clippy::too_many_arguments)]
pub fn manage_staging_speedup(
    mut time: ResMut<Time<Virtual>>,
    staging_query: Query<(), (With<StagingAttacker>, Without<Corpse>)>,
    activated_attackers: Query<&Team, (With<WaveGroup>, Without<StagingAttacker>, Without<Corpse>)>,
    wave_state: Option<Res<crate::game::resources::WaveState>>,
    config: Res<crate::config::GameConfig>,
    sp_state: Option<Res<State<crate::state::InGameState>>>,
    mp_state: Option<Res<State<crate::state::MultiplayerGameState>>>,
) {
    let has_staging = !staging_query.is_empty();

    let has_activated = activated_attackers
        .iter()
        .any(|team| *team == Team::Attackers);

    let waves_remaining = wave_state.map(|w| !w.waves_complete).unwrap_or(false);

    let speed_eligible = !has_activated && (has_staging || waves_remaining);

    // Any in-game menu overlay — SpellBook, CauldronMenu, Paused, etc. —
    // suppresses the speedup. Only `Running` (SP) or `Running` (MP) keeps it on.
    let in_menu_overlay = sp_state
        .as_ref()
        .is_some_and(|s| !matches!(*s.get(), crate::state::InGameState::Running))
        || mp_state
            .as_ref()
            .is_some_and(|s| !matches!(*s.get(), crate::state::MultiplayerGameState::Running));

    let should_speedup = speed_eligible && !in_menu_overlay;

    let current_speed = time.relative_speed_f64();
    let base_speed = config.game_speed as f64;
    let target_speed = if should_speedup {
        crate::game::constants::STAGING_SPEEDUP * base_speed
    } else {
        base_speed
    };

    if (current_speed - target_speed).abs() > 0.01 {
        time.set_relative_speed_f64(target_speed);
    }
}

/// Zeroes out targeting velocity for staging attackers so they only follow
/// the staging flow field. Without this, targeting systems point them at
/// enemies and the movement weighting lets targeting override the flow field.
pub fn suppress_staging_targeting(
    mut query: Query<&mut crate::game::units::components::TargetingVelocity, With<StagingAttacker>>,
) {
    for mut targeting in &mut query {
        if targeting.velocity != Vec3::ZERO || targeting.distance_to_target != f32::MAX {
            targeting.velocity = Vec3::ZERO;
            targeting.distance_to_target = f32::MAX;
        }
    }
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
