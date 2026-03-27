use std::cmp::Ordering;

use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use super::resources::ArcherAssets;
use super::styles::*;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::{
    calculate_defender_grid_position, cells_needed, distribute_units_to_cells, *,
};
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::plugin::GlobalAttackCycle;
use crate::game::units::components::{
    AttackTiming, BanishedModifier, CommanderAuraSpeedModifier, Corpse, Effectiveness,
    EliteSpeedBonus, FacingDirection, FlockingModifier, FlockingVelocity, HasteModifier, Health,
    Hitbox, MovementSpeed, PolymorphedModifier, RootedModifier, RoughTerrainModifier,
    FrozenSolidModifier, SickenedModifier, SleepModifier, Sleepwalking, SlowMovementModifier,
    TargetingVelocity, Team, Teleportable,
    TemporaryHitPoints, WalkingAnimation, apply_damage_to_unit,
};
use crate::game::units::infantry::components::DefendersActivated;
use crate::game::pathfinding::{StagingAttacker, WaveGroup};
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
    mut commands: Commands,
    attack_cycle: Res<GlobalAttackCycle>,
    archer_assets: Res<ArcherAssets>,
    mut archers: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut AttackTiming,
            &Effectiveness,
            Option<&SleepModifier>,
            Option<&BanishedModifier>,
            Has<crate::game::units::infantry::components::Retreating>,
            Has<StagingAttacker>,
            Has<WaveGroup>,
        ),
        (With<Archer>, Without<Corpse>),
    >,
    targets: Query<(Entity, &Transform, &Hitbox, &Team), (Without<Corpse>, Without<BanishedModifier>)>,
    mut health_query: Query<(&mut Health, Option<&mut TemporaryHitPoints>, Has<crate::game::units::shielder::components::ShielderDamageReduction>, Has<crate::game::units::assassin::Assassin>)>,
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
        sleeping,
        banished,
        is_retreating,
        has_staging,
        has_wave_group,
    ) in &mut archers
    {
        // Skip staging attackers (includes 1-frame delay before WaveGroup is added)
        if crate::game::units::systems::is_staging_attacker(archer_team, has_staging, has_wave_group) {
            continue;
        }

        // Skip attack if sleeping, banished, or retreating
        if sleeping.is_some() || banished.is_some() || is_retreating {
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
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal))
        {
            // Attack if we're in the unit's attack window
            if attack_timing.can_attack(current_time, last_time)
                && let Ok((mut target_health, mut temp_hp, has_shielder_reduction, is_assassin)) = health_query.get_mut(*target_entity)
            {
                // Apply effectiveness multiplier to melee damage
                let mut modified_damage = ARCHER_MELEE_DAMAGE * effectiveness.multiplier();
                if has_shielder_reduction {
                    modified_damage *= crate::game::units::shielder::constants::SHIELDER_DAMAGE_REDUCTION;
                }
                // Assassins take 50% less damage from archers (melee)
                if is_assassin {
                    modified_damage *= crate::game::units::assassin::constants::ARCHER_DAMAGE_REDUCTION;
                }
                apply_damage_to_unit(&mut target_health, temp_hp.as_deref_mut(), modified_damage);
                attack_timing.last_attack_time = Some(current_time);

                // Trigger melee attack animation
                commands.entity(archer_entity).insert(
                    crate::game::units::components::CombatAnimation::new_attack(
                        archer_assets.attacking_texture.clone(),
                        archer_assets.sprite_texture.clone(),
                    ),
                );
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
            Option<&SleepModifier>,
            Option<&BanishedModifier>,
            Has<crate::game::units::infantry::components::Retreating>,
            Option<&crate::game::units::wizard::spells::fog_cloud::components::BlindingMistDebuff>,
            Has<StagingAttacker>,
            Has<WaveGroup>,
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
            Has<crate::game::units::boss::components::Boss>,
        ),
        (Without<Corpse>, Without<BanishedModifier>),
    >,
    walls: Query<&WallOfStone>,
    concealing_veil_zones: Query<
        &crate::game::units::wizard::spells::fog_cloud::components::FogCloudZone,
        With<crate::game::units::wizard::spells::fog_cloud::components::ConcealingVeilZone>,
    >,
) {
    let wall_snapshot: Vec<_> = walls.iter().collect();

    // Collect concealing veil zone snapshots for ranged targeting checks
    let concealing_veil_snapshot: Vec<(Vec3, f32)> = concealing_veil_zones
        .iter()
        .map(|z| (z.origin, z.radius))
        .collect();

    for (
        archer_entity,
        archer_transform,
        _archer_hitbox,
        archer_team,
        attack_range,
        _attack_timing,
        mut movement_timer,
        sleeping,
        banished,
        is_retreating,
        blinding_mist,
        has_staging,
        has_wave_group,
    ) in archers.iter_mut()
    {
        // Skip staging attackers (includes 1-frame delay before WaveGroup is added)
        if crate::game::units::systems::is_staging_attacker(archer_team, has_staging, has_wave_group) {
            continue;
        }

        // Skip attack if sleeping, banished, or retreating
        if sleeping.is_some() || banished.is_some() || is_retreating {
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

        // Blinding Mist: halve max attack range when debuffed
        let effective_max_range = if let Some(debuff) = blinding_mist {
            attack_range.max_range * debuff.range_mult
        } else {
            attack_range.max_range
        };

        // Concealing Veil: skip check if archer is also inside a veil zone
        // (units inside fog together can still target each other)
        let archer_is_in_veil = !concealing_veil_snapshot.is_empty()
            && crate::game::units::wizard::spells::fog_cloud::systems::is_in_fog_zone(
                archer_transform.translation,
                &concealing_veil_snapshot,
            );

        // Find nearest enemy within ranged attack max_range
        // Exclude targets in melee with someone on the archer's own team
        let nearest_enemy = targets
            .iter()
            .filter(|(entity, _, team, _, in_melee, is_boss)| {
                // Skip self
                if *entity == archer_entity {
                    return false;
                }
                // Must be a valid enemy
                if !is_valid_target(archer_team, team) {
                    return false;
                }
                // Skip if target is in melee with archer's own team
                // (but always allow targeting the boss even when in melee)
                if !is_boss
                    && let Some(in_melee_component) = in_melee
                    && in_melee_component.0 == *archer_team
                {
                    return false;
                }
                true
            })
            .filter(|(_, transform, _, _, _, _)| {
                let distance = archer_transform.translation.distance(transform.translation);
                if distance > effective_max_range || distance < attack_range.min_range {
                    return false;
                }
                // Concealing Veil: units in fog can't be targeted by ranged attacks from outside
                if !archer_is_in_veil && !concealing_veil_snapshot.is_empty() {
                    if crate::game::units::wizard::spells::fog_cloud::systems::is_in_fog_zone(
                        transform.translation,
                        &concealing_veil_snapshot,
                    ) {
                        return false;
                    }
                }
                // Skip targets blocked by walls
                !WallOfStone::any_blocks_los(
                    &wall_snapshot,
                    archer_transform.translation,
                    transform.translation,
                )
            })
            .min_by(|a, b| {
                let dist_a = archer_transform.translation.distance(a.1.translation);
                let dist_b = archer_transform.translation.distance(b.1.translation);
                dist_a.partial_cmp(&dist_b).unwrap_or(Ordering::Equal)
            });

        if let Some((_, target_transform, _, _, _, _)) = nearest_enemy {
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

            // Trigger shooting animation
            commands.entity(archer_entity).insert(
                crate::game::units::components::CombatAnimation::new_shooting(
                    archer_assets.shooting_texture.clone(),
                    archer_assets.sprite_texture.clone(),
                ),
            );
        }
    }
}

/// Checks if a target is valid for the given team (same logic as combat system).
fn is_valid_target(source_team: &Team, target_team: &Team) -> bool {
    source_team.is_enemy(target_team)
}

/// Returns true if any wall is near the straight-line path between two points.
///
/// Unlike `line_segment_intersects` (which checks exact LOS), this uses a broader
/// check: if a wall's center projects onto the line segment and is within
/// (wall_extent + buffer) of the line, the wall is considered an obstruction.
/// This catches walls that force the flow field to detour even if they don't
/// block the geometric line-of-sight.
fn wall_near_approach_path(walls: &[&WallOfStone], from: Vec3, to: Vec3) -> bool {
    let line_dir = Vec3::new(to.x - from.x, 0.0, to.z - from.z);
    let line_len = line_dir.length();
    if line_len < 1.0 {
        return false;
    }
    let line_normalized = line_dir / line_len;

    for wall in walls {
        let to_wall = Vec3::new(wall.center.x - from.x, 0.0, wall.center.z - from.z);
        let projection = to_wall.dot(line_normalized);

        // Wall center must project between the two endpoints (with margin)
        if projection < 0.0 || projection > line_len {
            continue;
        }

        // Perpendicular distance from wall center to the line
        let closest_on_line = Vec3::new(
            from.x + line_normalized.x * projection,
            0.0,
            from.z + line_normalized.z * projection,
        );
        let perp_dist = Vec3::new(
            wall.center.x - closest_on_line.x,
            0.0,
            wall.center.z - closest_on_line.z,
        )
        .length();

        // Use the wall's largest extent plus a generous buffer so that walls
        // near (but not directly on) the LOS line still count as obstructions.
        let wall_extent = wall.half_length.max(wall.half_width);
        if perp_dist < wall_extent + WALL_APPROACH_PATH_BUFFER {
            return true;
        }
    }
    false
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

    // Calculate velocity needed to hit target at launch angle, accounting for height difference.
    // Projectile equation: h = d*tan(θ) - g*d² / (2*v²*cos²(θ))
    // Solving for v: v = (d/cos(θ)) * sqrt(g / (2*(d*tan(θ) - h)))
    let height_diff = target.y - origin.y;
    let tan_theta = launch_angle.tan();
    let cos_theta = launch_angle.cos();
    let denominator = 2.0 * (horizontal_distance * tan_theta - height_diff);

    let required_speed = if denominator > 0.1 {
        (horizontal_distance / cos_theta)
            * (ARROW_GRAVITY / denominator).sqrt()
            * power_multiplier
    } else {
        // Fallback for nearly-vertical or unreachable shots: use flat-ground formula
        let sin_2theta = (2.0 * launch_angle).sin();
        ((horizontal_distance * ARROW_GRAVITY) / sin_2theta).sqrt() * power_multiplier
    };

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
            Has<crate::game::units::shielder::components::ShielderDamageReduction>,
            Has<crate::game::units::assassin::Assassin>,
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
                commands.entity(arrow_entity).try_despawn();
                hit_wall = true;
                break;
            }
        }
        if hit_wall {
            continue;
        }

        // Ground collision
        if arrow_pos.y <= 0.0 {
            commands.entity(arrow_entity).try_despawn();
            continue;
        }

        // Unit collision (skip friendly fire)
        for (target_transform, hitbox, team, mut health, mut temp_hp, has_shielder_reduction, is_assassin) in &mut targets {
            // Skip same team
            if *team == arrow.source_team {
                continue;
            }

            let is_enemy = arrow.source_team.is_enemy(team);

            if !is_enemy {
                continue;
            }

            // Check collision (full 3D distance — arrows are true projectiles)
            let distance = arrow_pos.distance(target_transform.translation);
            if distance < hitbox.radius + ARROW_WIDTH {
                let mut damage = arrow.damage;
                if has_shielder_reduction {
                    damage *= crate::game::units::shielder::constants::SHIELDER_DAMAGE_REDUCTION;
                }
                // Assassins take 50% less damage from archers (arrows)
                if is_assassin {
                    damage *= crate::game::units::assassin::constants::ARCHER_DAMAGE_REDUCTION;
                }
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), damage);
                commands.entity(arrow_entity).try_despawn();
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
    all_units: Query<(Entity, &Transform, &Team), (Without<Corpse>, Without<BanishedModifier>, Without<StagingAttacker>)>,
    walls: Query<&WallOfStone>,
) {
    // Collect snapshot of all unit positions (excludes staging attackers)
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    // Collect wall snapshot for line-of-sight checks
    let wall_snapshot: Vec<_> = walls.iter().collect();

    // Update each archer's targeting velocity
    for (entity, transform, team, attack_range, mut targeting_velocity) in &mut archers {
        // Skip inactive defender archers (but always process attackers)
        if *team == Team::Defenders && !defenders_activated.active {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
            continue;
        }

        let pos = transform.translation;

        // Find nearest enemy in seek zone [min_range, seek_range]
        // Archers advance until enemies are within seek range, then stop.
        // They can still shoot up to max_range, but won't stop that far out.
        // Only count targets with clear line-of-sight (no walls blocking).
        let ranged_target = unit_snapshot
            .iter()
            .filter(|(other_entity, _, other_team)| {
                *other_entity != entity && team.is_enemy(other_team)
            })
            .filter_map(|&(_, target_pos, _)| {
                let dx = pos.x - target_pos.x;
                let dz = pos.z - target_pos.z;
                let dist = (dx * dx + dz * dz).sqrt();
                if dist >= attack_range.min_range && dist <= ARCHER_SEEK_RANGE {
                    // Check line-of-sight: skip if any wall blocks the shot
                    if WallOfStone::any_blocks_los(&wall_snapshot, pos, target_pos) {
                        None
                    } else {
                        Some((dist, target_pos))
                    }
                } else {
                    None
                }
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Find nearest enemy overall (fallback for melee or advancing)
        let nearest_enemy = unit_snapshot
            .iter()
            .filter(|(other_entity, _, other_team)| {
                *other_entity != entity && team.is_enemy(other_team)
            })
            .min_by(|a, b| {
                let dist_a = (pos.x - a.1.x).powi(2) + (pos.z - a.1.z).powi(2);
                let dist_b = (pos.x - b.1.x).powi(2) + (pos.z - b.1.z).powi(2);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        // Prefer ranged targets — only fall back to melee/advance if none in range
        if let Some((ranged_dist, target_pos)) = ranged_target {
            // If a wall is near the approach path (even if it doesn't block direct
            // LOS), keep advancing so the flow field routes the archer around it.
            // Otherwise enter shooting stance.
            targeting_velocity.velocity = if wall_near_approach_path(&wall_snapshot, pos, target_pos)
            {
                Vec3::new(target_pos.x - pos.x, 0.0, target_pos.z - pos.z).normalize_or_zero()
            } else {
                Vec3::ZERO
            };
            targeting_velocity.distance_to_target = ranged_dist;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
        } else if let Some(&(_, target_pos, enemy_team)) = nearest_enemy {
            let diff = target_pos - pos;
            let distance = (diff.x.powi(2) + diff.z.powi(2)).sqrt();
            targeting_velocity.distance_to_target = distance;

            let in_melee_range = distance < MELEE_SLOWDOWN_DISTANCE;
            if in_melee_range {
                commands
                    .entity(entity)
                    .insert(crate::game::units::components::InMelee(enemy_team));
                let direction = diff.normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
            } else {
                commands
                    .entity(entity)
                    .remove::<crate::game::units::components::InMelee>();
                // Beyond max range — advance toward enemy
                let direction = diff.normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
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
            Option<&SlowMovementModifier>,
            (
                Option<&CauldronSpeedModifier>,
                Option<&RootedModifier>,
                Option<&HasteModifier>,
                Option<&EliteSpeedBonus>,
            ),
            (
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&PolymorphedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&crate::game::units::components::Stunned>,
                &Team,
                Has<StagingAttacker>,
                Has<WaveGroup>,
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
        slow_modifier,
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (sleeping, sleepwalking, banished, polymorphed, sickened, frozen, stunned, team, has_staging, has_wave_group),
    ) in &mut archer_units
    {
        // CC'd units cannot move
        if crate::game::units::systems::is_cc_immobilized(rooted, sleeping, sleepwalking, banished, sickened, frozen, stunned) {
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
            slow_modifier.map(|m| m.modifier),
            cauldron_modifier.map(|m| m.0),
            haste_modifier.map(|m| m.modifier),
            elite_speed.map(|e| e.0),
        );

        // Archer-specific: Stop completely when in optimal shooting range (not in melee).
        // But keep moving if:
        //  - staging (needs to follow flow field to staging point)
        //  - standing on hazardous terrain (fire, spikes)
        //  - no target in range (needs to follow flow field back to spawn)
        //  - path is fully blocked (wall-attack system needs velocity)
        let is_staging = crate::game::units::systems::is_staging_attacker(team, has_staging, has_wave_group);
        if !is_staging
            && in_melee.is_none()
            && flow_field_velocity.terrain_cost <= 1.0
            && !flow_field_velocity.pathfinding_distance.is_infinite()
            && targeting_velocity.distance_to_target < f32::MAX
        {
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
    materials: &mut Assets<StandardMaterial>,
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

            let anim = WalkingAnimation::default();
            let material = crate::game::units::systems::create_default_sprite_material(
                materials,
                archer_assets.sprite_texture.clone(),
                DEFENDER_SPRITE_TINT,
            );

            commands
                .spawn((
                    Mesh3d(archer_assets.sprite_mesh.clone()),
                    MeshMaterial3d(material),
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
                    anim,
                    FacingDirection::default(),
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
    materials: &mut Assets<StandardMaterial>,
    unit_index: u32,
    _level: u32,
) {
    let (spawn_x, spawn_z) = attacker_spawn_position(unit_index, ARCHER_SPAWN_DEPTH_OFFSET);
    let (final_x, final_z) = random_position_in_cell(spawn_x, spawn_z);

    let hitbox = Hitbox::new(ARCHER_RADIUS, ATTACKER_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;

    let anim = WalkingAnimation::default();
    let material = crate::game::units::systems::create_default_sprite_material(
        materials,
        archer_assets.sprite_texture.clone(),
        ATTACKER_SPRITE_TINT,
    );

    commands
        .spawn((
            Mesh3d(archer_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
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
            anim,
            FacingDirection::default(),
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
}
