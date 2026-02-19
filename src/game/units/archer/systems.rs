use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use super::resources::ArcherAssets;
use super::styles::*;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::{
    calculate_defender_grid_position, calculate_grid_cell_position, calculate_spawn_cells,
    calculate_total_archers, calculate_total_infantry, cells_needed, distribute_units_to_cells, *,
};
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::plugin::GlobalAttackCycle;
use crate::game::units::components::{
    AttackTiming, BanishedModifier, CommanderAuraSpeedModifier, Corpse, Effectiveness,
    EliteSpeedBonus, FlockingModifier, FlockingVelocity, FrostSlowModifier, GreaseSlipModifier,
    HasteModifier, Health, Hitbox, MesmerizedModifier, MovementSpeed, PolymorphedModifier,
    RootedModifier, RoughTerrainModifier, SleepModifier, SpikeGrowthSlowModifier,
    TargetingVelocity, Team, Teleportable, TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::infantry::components::DefendersActivated;
use crate::game::units::random_position_in_cell;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;

/// Updates archer movement timers to track time since stopped moving.
pub fn update_archer_movement_timers(
    time: Res<Time>,
    mut archers: Query<(&Velocity, &mut ArcherMovementTimer), With<Archer>>,
) {
    let delta = time.delta_secs();
    for (velocity, mut timer) in &mut archers {
        // Check if archer is moving (velocity threshold - very low to catch nearly stationary archers)
        let is_moving = velocity.x.abs() > 0.1 || velocity.z.abs() > 0.1;

        if is_moving {
            // Archer is moving
            timer.time_since_stopped = 0.0;
            timer.was_moving = true;
        } else if timer.was_moving {
            // Just stopped moving
            timer.time_since_stopped = 0.0;
            timer.was_moving = false;
        } else {
            // Stationary - accumulate time
            timer.time_since_stopped += delta;
        }

        // Always tick attack cooldown
        timer.time_since_last_attack += delta;
    }
}

/// Archer melee combat system (used when enemies are in melee range).
/// Archers deal reduced damage in melee compared to infantry.
#[allow(clippy::type_complexity)]
pub fn archer_melee_combat(
    attack_cycle: Res<GlobalAttackCycle>,
    mut archers: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut AttackTiming,
            &Effectiveness,
            Option<&MesmerizedModifier>,
            Option<&SleepModifier>,
            Option<&BanishedModifier>,
        ),
        (With<Archer>, Without<Corpse>),
    >,
    targets: Query<(Entity, &Transform, &Hitbox, &Team), Without<Corpse>>,
    mut health_query: Query<(&mut Health, Option<&mut TemporaryHitPoints>)>,
) {
    let current_time = attack_cycle.current_time;
    let last_time = (current_time - APPROX_FRAME_TIME).max(0.0);

    // Collect snapshot of all targets
    let targets_snapshot: Vec<_> = targets
        .iter()
        .map(|(entity, transform, hitbox, team)| (entity, transform.translation, *hitbox, *team))
        .collect();

    for (
        archer_entity,
        archer_transform,
        archer_hitbox,
        archer_team,
        mut attack_timing,
        effectiveness,
        mesmerized,
        sleeping,
        banished,
    ) in &mut archers
    {
        // Skip attack if mesmerized, sleeping, or banished
        if mesmerized.is_some() || sleeping.is_some() || banished.is_some() {
            continue;
        }

        // Find nearest enemy within melee range
        if let Some((target_entity, _, _)) = targets_snapshot
            .iter()
            .filter(|(entity, _, _, team)| {
                *entity != archer_entity && is_valid_target(archer_team, team)
            })
            .filter_map(|(entity, target_pos, target_hitbox, _)| {
                // Calculate distance on XZ plane only (ignore Y axis for attack range)
                let dx = archer_transform.translation.x - target_pos.x;
                let dz = archer_transform.translation.z - target_pos.z;
                let distance = (dx * dx + dz * dz).sqrt();
                let melee_range =
                    (archer_hitbox.radius + target_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;
                if distance <= melee_range {
                    Some((entity, target_pos, distance))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
        {
            // Attack if we're in the unit's attack window
            if attack_timing.can_attack(current_time, last_time)
                && let Ok((mut target_health, mut temp_hp)) = health_query.get_mut(*target_entity)
            {
                // Apply effectiveness multiplier to melee damage
                let modified_damage = ARCHER_MELEE_DAMAGE * effectiveness.multiplier();
                apply_damage_to_unit(&mut target_health, temp_hp.as_deref_mut(), modified_damage);
                attack_timing.last_attack_time = Some(current_time);
            }
        }
    }
}

/// Archer ranged combat system that spawns arrows instead of dealing direct damage.
/// Only fires if no melee targets are available.
#[allow(clippy::type_complexity)]
pub fn archer_ranged_combat(
    mut commands: Commands,
    archer_assets: Res<ArcherAssets>,
    mut archers: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &AttackRange,
            &mut AttackTiming,
            &mut ArcherMovementTimer,
            Option<&MesmerizedModifier>,
            Option<&SleepModifier>,
            Option<&BanishedModifier>,
        ),
        (With<Archer>, Without<Corpse>),
    >,
    targets: Query<
        (
            Entity,
            &Transform,
            &Team,
            &Hitbox,
            Option<&crate::game::units::components::InMelee>,
        ),
        Without<Corpse>,
    >,
) {
    for (
        archer_entity,
        archer_transform,
        _archer_hitbox,
        archer_team,
        attack_range,
        _attack_timing,
        mut movement_timer,
        mesmerized,
        sleeping,
        banished,
    ) in archers.iter_mut()
    {
        // Skip attack if mesmerized, sleeping, or banished
        if mesmerized.is_some() || sleeping.is_some() || banished.is_some() {
            continue;
        }

        // Check if enough time has passed since stopping to attack
        if !movement_timer.can_attack(ARCHER_ATTACK_DELAY_AFTER_MOVEMENT) {
            continue;
        }

        // Check attack cooldown
        let attack_cooldown = ATTACK_CYCLE_DURATION * ARCHER_ATTACK_COOLDOWN_MULTIPLIER;
        if movement_timer.time_since_last_attack < attack_cooldown {
            continue;
        }

        // Find nearest enemy within ranged attack max_range
        // Exclude targets in melee with someone on the archer's own team
        let nearest_enemy = targets
            .iter()
            .filter(|(entity, _, team, _, in_melee)| {
                // Skip self
                if *entity == archer_entity {
                    return false;
                }
                // Must be a valid enemy
                if !is_valid_target(archer_team, team) {
                    return false;
                }
                // Skip if target is in melee with archer's own team
                if let Some(in_melee_component) = in_melee
                    && in_melee_component.0 == *archer_team
                {
                    return false;
                }
                true
            })
            .filter(|(_, transform, _, _, _)| {
                let distance = archer_transform.translation.distance(transform.translation);
                distance <= attack_range.max_range && distance >= attack_range.min_range
            })
            .min_by(|a, b| {
                let dist_a = archer_transform.translation.distance(a.1.translation);
                let dist_b = archer_transform.translation.distance(b.1.translation);
                dist_a.partial_cmp(&dist_b).unwrap()
            });

        if let Some((_, target_transform, _, _, _)) = nearest_enemy {
            // Spawn arrow projectile directly above the archer
            spawn_arrow(
                &mut commands,
                &archer_assets,
                archer_transform.translation + Vec3::Y * 10.0,
                target_transform.translation,
                *archer_team,
            );
            // Reset attack cooldown
            movement_timer.time_since_last_attack = 0.0;
        }
    }
}

/// Checks if a target is valid for the given team (same logic as combat system).
fn is_valid_target(source_team: &Team, target_team: &Team) -> bool {
    match (source_team, target_team) {
        (Team::Undead, Team::Undead) => false, // Undead don't attack each other
        (Team::Undead, _) => true,             // Undead attack living
        (_, Team::Undead) => true,             // Living attack undead
        _ => target_team != source_team,       // Normal team logic
    }
}

/// Spawns an arrow projectile from archer toward target.
fn spawn_arrow(
    commands: &mut Commands,
    archer_assets: &ArcherAssets,
    origin: Vec3,
    target: Vec3,
    source_team: Team,
) {
    // Calculate horizontal direction and distance
    let horizontal_diff = Vec3::new(target.x - origin.x, 0.0, target.z - origin.z);
    let horizontal_distance = horizontal_diff.length();

    // Avoid division by zero
    if horizontal_distance < 0.1 {
        return;
    }

    let horizontal_direction = horizontal_diff.normalize();

    // Add random variations for realism
    let mut rng = rand::thread_rng();

    // Random power variation (±5%)
    let power_multiplier = 1.0 + rng.gen_range(-ARROW_POWER_VARIATION..ARROW_POWER_VARIATION);

    // Random angle variation (±1 degree)
    let angle_offset = rng.gen_range(-ARROW_ANGLE_VARIATION_DEGREES..ARROW_ANGLE_VARIATION_DEGREES);
    let launch_angle = (ARROW_LAUNCH_ANGLE_DEGREES + angle_offset).to_radians();

    // Calculate velocity needed to hit target at launch angle (flat ground)
    // Range = v^2 * sin(2*theta) / g
    // Solving for v: v = sqrt(Range * g / sin(2*theta))
    let sin_2theta = (2.0 * launch_angle).sin();
    let required_speed =
        ((horizontal_distance * ARROW_GRAVITY) / sin_2theta).sqrt() * power_multiplier;

    // Calculate velocity components
    let horizontal_velocity = horizontal_direction * required_speed * launch_angle.cos();
    let vertical_velocity = required_speed * launch_angle.sin();

    let velocity = Vec3::new(
        horizontal_velocity.x,
        vertical_velocity,
        horizontal_velocity.z,
    );

    // Spawn arrow using pre-loaded assets
    commands.spawn((
        Mesh3d(archer_assets.arrow_mesh.clone()),
        MeshMaterial3d(archer_assets.arrow_material.clone()),
        Transform::from_translation(origin),
        Arrow {
            velocity,
            damage: ARCHER_ATTACK_DAMAGE,
            source_team,
        },
        crate::game::components::Billboard,
        OnGameplayScreen,
    ));
}

/// Updates arrow positions with gravity.
pub fn move_arrows(time: Res<Time>, mut arrows: Query<(&mut Transform, &mut Arrow)>) {
    let delta = time.delta_secs();
    for (mut transform, mut arrow) in &mut arrows {
        // Apply gravity
        arrow.velocity.y -= ARROW_GRAVITY * delta;

        // Update position
        transform.translation += arrow.velocity * delta;
    }
}

/// Checks arrow collisions with units and ground.
pub fn check_arrow_collisions(
    mut commands: Commands,
    arrows: Query<(Entity, &Transform, &Arrow)>,
    mut targets: Query<
        (
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<Corpse>,
    >,
    walls: Query<&WallOfStone>,
) {
    #[allow(clippy::significant_drop_in_scrutinee)]
    for (arrow_entity, arrow_transform, arrow) in &arrows {
        let arrow_pos = arrow_transform.translation;

        // Wall collision
        let mut hit_wall = false;
        for wall in &walls {
            if wall.contains_point_xz(arrow_pos) && arrow_pos.y <= wall.height {
                commands.entity(arrow_entity).despawn();
                hit_wall = true;
                break;
            }
        }
        if hit_wall {
            continue;
        }

        // Ground collision
        if arrow_pos.y <= 0.0 {
            commands.entity(arrow_entity).despawn();
            continue;
        }

        // Unit collision (skip friendly fire)
        for (target_transform, hitbox, team, mut health, mut temp_hp) in &mut targets {
            // Skip same team
            if *team == arrow.source_team {
                continue;
            }

            // Check if enemy (using same logic as combat system)
            let is_enemy = match (arrow.source_team, *team) {
                (Team::Undead, Team::Undead) => false,
                (Team::Undead, _) => true,
                (_, Team::Undead) => true,
                _ => *team != arrow.source_team,
            };

            if !is_enemy {
                continue;
            }

            // Check collision
            let distance = arrow_pos.distance(target_transform.translation);
            if distance < hitbox.radius + ARROW_WIDTH {
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), arrow.damage);
                commands.entity(arrow_entity).despawn();
                break;
            }
        }
    }
}

/// Updates archer targeting velocity based on attack range.
///
/// Archers stop moving when in optimal range and retreat when enemies are too close.
/// Also sets InMelee component if an enemy is within melee range.
/// Defender archers are gated by the DefendersActivated resource.
pub fn update_archer_targeting(
    defenders_activated: Res<DefendersActivated>,
    mut commands: Commands,
    mut archers: Query<
        (
            Entity,
            &Transform,
            &Team,
            &AttackRange,
            &mut crate::game::units::components::TargetingVelocity,
        ),
        (With<Archer>, Without<Corpse>),
    >,
    all_units: Query<(Entity, &Transform, &Team), Without<Corpse>>,
) {
    // Collect snapshot of all unit positions
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    // Update each archer's targeting velocity
    for (entity, transform, team, attack_range, mut targeting_velocity) in &mut archers {
        // Skip inactive defender archers (but always process attackers)
        if *team == Team::Defenders && !defenders_activated.active {
            *targeting_velocity = TargetingVelocity::default();
            continue;
        }
        // Find nearest enemy
        let nearest_enemy = unit_snapshot
            .iter()
            .filter(|(other_entity, _, other_team)| {
                *other_entity != entity
                    && match (*team, other_team) {
                        (Team::Undead, Team::Undead) => false,
                        (Team::Undead, _) => true,
                        (_, Team::Undead) => true,
                        _ => *other_team != *team,
                    }
            })
            .min_by(|a, b| {
                let dist_a = (transform.translation.x - a.1.x).powi(2)
                    + (transform.translation.z - a.1.z).powi(2);
                let dist_b = (transform.translation.x - b.1.x).powi(2)
                    + (transform.translation.z - b.1.z).powi(2);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        // Set targeting velocity based on range to enemy
        if let Some(&(_, target_pos, enemy_team)) = nearest_enemy {
            let diff = target_pos - transform.translation;
            let distance = (diff.x.powi(2) + diff.z.powi(2)).sqrt();

            // Store distance for formation weighting
            targeting_velocity.distance_to_target = distance;

            // Set InMelee component if enemy is in melee range
            let in_melee_range = distance < MELEE_SLOWDOWN_DISTANCE;
            if in_melee_range {
                commands
                    .entity(entity)
                    .insert(crate::game::units::components::InMelee(enemy_team));
            } else {
                commands
                    .entity(entity)
                    .remove::<crate::game::units::components::InMelee>();
            }

            if in_melee_range {
                // IN MELEE - move toward enemy for melee combat
                let direction = diff.normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
            } else if distance > attack_range.max_range {
                // TOO FAR - advance toward enemy
                let direction = diff.normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
            } else {
                // IN RANGE - stop moving and shoot
                targeting_velocity.velocity = Vec3::ZERO;
            }
        } else {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
        }
    }
}

/// Archer-specific movement system.
///
/// Uses acceleration-based physics with maximum speed capping.
/// TargetingVelocity and FlockingVelocity are treated as acceleration forces.
/// Units slow down when in melee to prevent erratic movement.
#[allow(clippy::type_complexity)]
pub fn archer_movement(
    time: Res<Time>,
    mut archer_units: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &Effectiveness,
            &TargetingVelocity,
            &crate::game::units::components::FlockingVelocity,
            &FlowFieldVelocity,
            Option<&crate::game::units::components::InMelee>,
            Option<&CommanderAuraSpeedModifier>,
            Option<&RoughTerrainModifier>,
            (Option<&FrostSlowModifier>, Option<&SpikeGrowthSlowModifier>),
            (
                Option<&CauldronSpeedModifier>,
                Option<&RootedModifier>,
                Option<&HasteModifier>,
                Option<&EliteSpeedBonus>,
            ),
            (
                Option<&MesmerizedModifier>,
                Option<&SleepModifier>,
                Option<&BanishedModifier>,
                Option<&GreaseSlipModifier>,
                Option<&PolymorphedModifier>,
            ),
        ),
        With<Archer>,
    >,
) {
    // Process each archer unit
    for (
        mut velocity,
        mut acceleration,
        movement_speed,
        effectiveness,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        in_melee,
        aura_modifier,
        terrain_modifier,
        (frost_modifier, spike_growth_modifier),
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (mesmerized, sleeping, banished, grease, polymorphed),
    ) in &mut archer_units
    {
        // CC'd units cannot move
        if rooted.is_some() || mesmerized.is_some() || sleeping.is_some() || banished.is_some() {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

        // Polymorphed units wander randomly
        if polymorphed.is_some() {
            let angle = (time.elapsed_secs() * 0.5 + velocity.x.to_bits() as f32).sin()
                * std::f32::consts::TAU;
            velocity.x = angle.cos() * 20.0;
            velocity.z = angle.sin() * 20.0;
            continue;
        }

        // Use shared weighted movement function
        crate::game::units::systems::calculate_weighted_movement(
            &time,
            &mut velocity,
            &mut acceleration,
            movement_speed.0,
            effectiveness,
            targeting_velocity,
            flocking_velocity,
            flow_field_velocity,
            in_melee.is_some(),
            aura_modifier.map(|m| m.0),
            terrain_modifier.map(|m| m.0),
            frost_modifier.map(|m| m.modifier),
            spike_growth_modifier.map(|m| m.modifier),
            cauldron_modifier.map(|m| m.0),
            haste_modifier.map(|m| m.modifier),
            elite_speed.map(|e| e.0),
            grease.map(|g| g.modifier),
        );

        // Archer-specific: Stop completely when in optimal shooting range (not in melee)
        // But keep moving if standing on hazardous terrain (fire, spikes)
        if in_melee.is_none() && flow_field_velocity.terrain_cost <= 1.0 {
            let targeting_is_zero = targeting_velocity.velocity.length_squared() < 0.01;
            if targeting_is_zero {
                // Override velocity and acceleration to completely stop archer when in shooting stance
                velocity.x = 0.0;
                velocity.z = 0.0;
                acceleration.x = 0.0;
                acceleration.z = 0.0;
            }
        }
    }
}

/// Spawns a single defender archer unit at a specific index.
/// Used for progressive loading.
pub(in crate::game) fn spawn_single_defender_archer(
    commands: &mut Commands,
    archer_assets: &ArcherAssets,
    unit_index: u32,
) {
    // Calculate where infantry spawned to determine archer row
    let infantry_cells = cells_needed(INITIAL_DEFENDER_COUNT);
    let infantry_rows = infantry_cells.div_ceil(DEFENDER_GRID_COLS);
    // Infantry start at row (ROWS-1) and fill `infantry_rows` rows, ending at (ROWS-1-infantry_rows+1)
    // Archers go one row lower than that
    let last_infantry_row = DEFENDER_GRID_ROWS.saturating_sub(infantry_rows);
    let archer_row = last_infantry_row.saturating_sub(1);

    let archer_cells_needed = cells_needed(INITIAL_ARCHER_DEFENDER_COUNT);
    let units_per_cell = distribute_units_to_cells(INITIAL_ARCHER_DEFENDER_COUNT);

    // Calculate which cell this unit belongs to
    let mut units_counted = 0;
    for cell_idx in 0..archer_cells_needed.min(DEFENDER_GRID_COLS) {
        let units_in_this_cell = units_per_cell[cell_idx as usize];
        if unit_index < units_counted + units_in_this_cell {
            // This unit goes in this cell
            let (spawn_x, spawn_z) = calculate_defender_grid_position(archer_row, cell_idx);
            let (final_x, final_z) = random_position_in_cell(spawn_x, spawn_z);

            let hitbox = Hitbox::new(ARCHER_RADIUS, DEFENDER_HITBOX_HEIGHT);
            let spawn_y = hitbox.height / 2.0 + 1.0;

            commands
                .spawn((
                    Mesh3d(archer_assets.mesh.clone()),
                    MeshMaterial3d(archer_assets.defender_material.clone()),
                    Transform::from_xyz(final_x, spawn_y, final_z),
                    Velocity::default(),
                    Acceleration::new(),
                    hitbox,
                    Health::new(UNIT_HEALTH),
                    MovementSpeed(ARCHER_MOVEMENT_SPEED),
                    AttackTiming::new(),
                    Effectiveness::new(),
                    Team::Defenders,
                    Archer,
                ))
                .insert((
                    AttackRange {
                        min_range: ARCHER_MIN_RANGE,
                        max_range: ARCHER_MAX_RANGE,
                    },
                    ArcherMovementTimer::new(),
                    TargetingVelocity::default(),
                    FlockingVelocity::default(),
                    FlowFieldVelocity::default(),
                    FlowFieldInfluence::Defender {
                        spawn_pos: Vec2::new(spawn_x, spawn_z),
                    },
                    FlockingModifier::new(1.0, 1.0, 0.0),
                    Teleportable,
                    Billboard,
                    OnGameplayScreen,
                ));
            return;
        }
        units_counted += units_in_this_cell;
    }
}

/// Spawns a single attacker archer unit at a specific index.
/// Used for progressive loading.
pub(in crate::game) fn spawn_single_attacker_archer(
    commands: &mut Commands,
    archer_assets: &ArcherAssets,
    unit_index: u32,
    level: u32,
) {
    let total_archers = calculate_total_archers(level);
    let total_infantry = calculate_total_infantry(level);
    let infantry_cells_needed = cells_needed(total_infantry);
    let archer_cells_needed = cells_needed(total_archers);

    let (_, archer_cells) = calculate_spawn_cells(infantry_cells_needed, archer_cells_needed);
    let units_per_cell = distribute_units_to_cells(total_archers);

    // Calculate which cell this unit belongs to
    let mut units_counted = 0;
    for (cell_idx, (row, col)) in archer_cells.iter().enumerate() {
        if cell_idx >= units_per_cell.len() {
            break;
        }
        let units_in_this_cell = units_per_cell[cell_idx];
        if unit_index < units_counted + units_in_this_cell {
            // This unit goes in this cell
            let (spawn_x, spawn_z) = calculate_grid_cell_position(*row, *col);
            let (final_x, final_z) = random_position_in_cell(spawn_x, spawn_z);

            let hitbox = Hitbox::new(ARCHER_RADIUS, ATTACKER_HITBOX_HEIGHT);
            let spawn_y = hitbox.height / 2.0 + 1.0;

            commands
                .spawn((
                    Mesh3d(archer_assets.mesh.clone()),
                    MeshMaterial3d(archer_assets.attacker_material.clone()),
                    Transform::from_xyz(final_x, spawn_y, final_z),
                    Velocity::default(),
                    Acceleration::new(),
                    hitbox,
                    Health::new(UNIT_HEALTH),
                    MovementSpeed(ARCHER_MOVEMENT_SPEED),
                    AttackTiming::new(),
                    Effectiveness::new(),
                    Team::Attackers,
                    Archer,
                ))
                .insert((
                    AttackRange {
                        min_range: ARCHER_MIN_RANGE,
                        max_range: ARCHER_MAX_RANGE,
                    },
                    ArcherMovementTimer::new(),
                    TargetingVelocity::default(),
                    FlockingVelocity::default(),
                    FlowFieldVelocity::default(),
                    FlowFieldInfluence::Attacker,
                    Teleportable,
                    Billboard,
                    OnGameplayScreen,
                ));
            return;
        }
        units_counted += units_in_this_cell;
    }
}
