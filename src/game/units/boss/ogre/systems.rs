use std::collections::HashSet;

use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::resources::OgreAssets;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity, StagingAttacker};
use crate::game::units::boss::components::Boss;
use crate::game::units::boss::lich::Lich;
use crate::game::units::components::{
    AttackTiming, BanishedModifier, CommanderAuraSpeedModifier, Corpse, DamageMultiplier,
    Effectiveness, EliteSpeedBonus, FlockingModifier, FlockingVelocity, HasteModifier, Health,
    Hitbox, InMelee, Knockback, MovementSpeed, OriginalMaterial, PolymorphedModifier,
    FrozenSolidModifier, RootedModifier, RoughTerrainModifier, SickenedModifier, SleepModifier,
    Sleepwalking, SlowMovementModifier,
    TargetingVelocity, Team, Teleportable, TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::random_position_in_cell;
use crate::game::units::brute::components::RockThrowCooldown;
use crate::game::terrain::boulder::constants::{ROCK_THROW_COOLDOWN, ROCK_THROW_RANGE};
use crate::game::terrain::boulder::messages::BoulderThrownMessage;

/// Spawns the ogre at one of the tunnel spawn points.
pub fn spawn_ogre(mut commands: Commands, ogre_assets: Res<OgreAssets>) {
    let (spawn_x, spawn_z) = attacker_spawn_position(0, 0.0);
    let (final_x, final_z) = random_position_in_cell(spawn_x, spawn_z);

    let hitbox = Hitbox::new(OGRE_RADIUS, OGRE_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + (OGRE_ELLIPSE_DEPTH / 2.0) + 1.0;

    // Initial velocity toward castle
    let to_center = Vec3::new(
        WIZARD_POSITION.x - final_x,
        0.0,
        WIZARD_POSITION.z - final_z,
    );
    let initial_velocity = to_center.normalize_or_zero() * OGRE_MOVEMENT_SPEED;

    commands
        .spawn((
            // Rendering
            Mesh3d(ogre_assets.mesh.clone()),
            MeshMaterial3d(ogre_assets.material_phase0.clone()),
            Transform::from_xyz(final_x, spawn_y, final_z),
            // Physics
            Velocity {
                x: initial_velocity.x,
                z: initial_velocity.z,
                ..default()
            },
            Acceleration::new(),
            // Core
            hitbox,
            Health::new(OGRE_HEALTH),
            MovementSpeed(OGRE_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Attackers,
            Boss,
        ))
        .insert((
            OgreEnrageState::new(),
            OgreAttackCooldown::new(),
            OgreChargeState::Idle {
                cooldown: OGRE_CHARGE_COOLDOWN,
            },
            MeleeDamageReduction {
                multiplier: OGRE_MELEE_DAMAGE_REDUCTION,
            },
            // Movement systems
            TargetingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Attacker,
            DamageMultiplier(OGRE_DAMAGE_MULTIPLIER),
            FlockingVelocity::default(),
            FlockingModifier::new(0.0, 0.0, 0.0),
            CommanderAuraSpeedModifier(0.0),
            RoughTerrainModifier(0.0),
            Teleportable,
            Billboard,
            OnGameplayScreen,
        ))
        .insert(RockThrowCooldown::new(8.0));
}

/// Updates ogre targeting velocity toward nearest enemy.
pub fn update_ogre_targeting(
    mut commands: Commands,
    mut bosses: Query<(Entity, &Transform, &Team, &mut TargetingVelocity), (With<Boss>, Without<Lich>)>,
    all_units: Query<(Entity, &Transform, &Team), (Without<Boss>, Without<Corpse>, Without<BanishedModifier>, Without<StagingAttacker>)>,
) {
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, boss_transform, boss_team, mut targeting) in &mut bosses {
        crate::game::units::systems::update_melee_unit_targeting(
            &unit_snapshot,
            entity,
            boss_transform,
            *boss_team,
            &mut targeting,
            &mut commands,
            None,
        );
    }
}

/// Ogre melee combat system — runs on its own cooldown timer (not the global attack cycle).
/// Finds nearest enemy in melee range, deals flat damage, and applies a tumbling
/// knockback effect to all nearby enemies.
pub fn ogre_combat(
    time: Res<Time>,
    mut commands: Commands,
    mut bosses: Query<
        (Entity, &Transform, &Hitbox, &Team, &mut OgreAttackCooldown, &OgreChargeState),
        (With<Boss>, Without<Corpse>),
    >,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (Without<Boss>, Without<Corpse>, Without<BanishedModifier>),
    >,
) {
    let delta = time.delta_secs();

    for (boss_entity, boss_transform, boss_hitbox, boss_team, mut attack_cooldown, charge_state) in &mut bosses {
        // Skip normal melee attacks during charge
        if charge_state.is_movement_locked() {
            continue;
        }

        attack_cooldown.tick(delta);

        if !attack_cooldown.is_ready() {
            continue;
        }

        // Find nearest enemy in melee range
        let boss_pos = boss_transform.translation;
        let mut has_target = false;

        // First pass: check if any enemy is in melee range
        for (entity, target_transform, target_hitbox, team, _, _) in &targets {
            if entity == boss_entity {
                continue;
            }
            if !boss_team.is_enemy(team) {
                continue;
            }

            let dx = boss_pos.x - target_transform.translation.x;
            let dz = boss_pos.z - target_transform.translation.z;
            let distance = (dx * dx + dz * dz).sqrt();
            let attack_range =
                (boss_hitbox.radius + target_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;
            if distance <= attack_range {
                has_target = true;
                break;
            }
        }

        if !has_target {
            continue;
        }

        // Reset cooldown — ogre attacked
        attack_cooldown.reset(OGRE_ATTACK_COOLDOWN);

        // Second pass: apply damage and knockback to all enemies within ogre melee reach
        for (entity, target_transform, target_hitbox, team, mut health, mut temp_hp) in &mut targets
        {
            if entity == boss_entity {
                continue;
            }
            let is_enemy = boss_team.is_enemy(team);
            if !is_enemy {
                continue;
            }

            let target_pos = target_transform.translation;
            let dx = target_pos.x - boss_pos.x;
            let dz = target_pos.z - boss_pos.z;
            let distance = (dx * dx + dz * dz).sqrt();

            // Hit all enemies within boss radius + their hitbox (melee reach)
            if distance > boss_hitbox.radius + target_hitbox.radius {
                continue;
            }

            // Apply damage
            apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), OGRE_ATTACK_DAMAGE);

            // Apply tumbling knockback (decays over time)
            let direction = if distance > 0.1 {
                Vec3::new(dx, 0.0, dz)
            } else {
                Vec3::X
            };
            commands.entity(entity).insert(Knockback::new(
                direction,
                OGRE_MELEE_KNOCKBACK_SPEED,
                OGRE_MELEE_KNOCKBACK_DURATION,
            ));
        }
    }
}

/// Ogre movement system using weighted velocities.
/// Feeds enrage speed bonus through the haste parameter.
#[allow(clippy::type_complexity)]
pub fn ogre_movement(
    time: Res<Time>,
    mut bosses: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &Effectiveness,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            &OgreEnrageState,
            &OgreChargeState,
            Option<&InMelee>,
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
            ),
        ),
        With<Boss>,
    >,
) {
    for (
        mut velocity,
        mut acceleration,
        movement_speed,
        effectiveness,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        enrage_state,
        charge_state,
        in_melee,
        aura_modifier,
        terrain_modifier,
        slow_modifier,
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (sleeping, sleepwalking, banished, polymorphed, sickened, frozen, stunned),
    ) in &mut bosses
    {
        // Freeze normal movement during charge phases; also zero acceleration
        // so external forces (e.g. black hole gravity) don't drift the ogre off course
        if charge_state.is_movement_locked() {
            velocity.x = 0.0;
            velocity.z = 0.0;
            acceleration.x = 0.0;
            acceleration.z = 0.0;
            continue;
        }

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

        // Combine haste modifier with enrage speed bonus
        let combined_haste =
            Some(haste_modifier.map(|m| m.modifier).unwrap_or(0.0) + enrage_state.speed_bonus);

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
            combined_haste,
            elite_speed.map(|e| e.0),
        );
    }
}

/// Updates the ogre's enrage state based on HP thresholds.
/// Swaps material to match the current enrage phase.
#[allow(clippy::type_complexity)]
pub fn update_enrage_state(
    mut bosses: Query<
        (
            &Health,
            &mut OgreEnrageState,
            &mut DamageMultiplier,
            &mut MeshMaterial3d<StandardMaterial>,
            Option<&mut OriginalMaterial>,
        ),
        With<Boss>,
    >,
    ogre_assets: Res<OgreAssets>,
) {
    for (health, mut enrage, mut damage_mult, mut mesh_material, original_material) in &mut bosses {
        let hp_ratio = health.current / health.max;

        let new_phase = if hp_ratio <= ENRAGE_PHASE_3_THRESHOLD {
            3
        } else if hp_ratio <= ENRAGE_PHASE_2_THRESHOLD {
            2
        } else if hp_ratio <= ENRAGE_PHASE_1_THRESHOLD {
            1
        } else {
            0
        };

        if new_phase != enrage.phase {
            enrage.phase = new_phase;

            // Update bonuses
            match new_phase {
                1 => {
                    enrage.speed_bonus = ENRAGE_1_SPEED_BONUS;
                    enrage.damage_bonus = ENRAGE_1_DAMAGE_BONUS;
                }
                2 => {
                    enrage.speed_bonus = ENRAGE_2_SPEED_BONUS;
                    enrage.damage_bonus = ENRAGE_2_DAMAGE_BONUS;
                }
                3 => {
                    enrage.speed_bonus = ENRAGE_3_SPEED_BONUS;
                    enrage.damage_bonus = ENRAGE_3_DAMAGE_BONUS;
                }
                _ => {
                    enrage.speed_bonus = 0.0;
                    enrage.damage_bonus = 0.0;
                }
            }

            // Update damage multiplier (base + enrage bonus)
            damage_mult.0 = OGRE_DAMAGE_MULTIPLIER + enrage.damage_bonus;

            // Pick the phase material
            let phase_material = match new_phase {
                1 => ogre_assets.material_phase1.clone(),
                2 => ogre_assets.material_phase2.clone(),
                3 => ogre_assets.material_phase3.clone(),
                _ => ogre_assets.material_phase0.clone(),
            };

            // If OriginalMaterial is present (spell effect active), update that
            // so the correct enrage color restores when the effect ends.
            if let Some(mut orig) = original_material {
                orig.0 = phase_material;
            } else {
                mesh_material.0 = phase_material;
            }
        }
    }
}

/// Ogre charge ability — state machine handling targeting, telegraph, dash, and recovery.
#[allow(clippy::type_complexity)]
pub fn ogre_charge_system(
    time: Res<Time>,
    mut commands: Commands,
    ogre_assets: Res<OgreAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut bosses: Query<
        (
            Entity,
            &mut Transform,
            &Team,
            &mut OgreChargeState,
            (
                Option<&RootedModifier>,
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&crate::game::units::components::Stunned>,
            ),
        ),
        (With<Boss>, Without<Corpse>),
    >,
    potential_targets: Query<
        (Entity, &Transform, &Team),
        (Without<Boss>, Without<Corpse>, Without<BanishedModifier>, Without<StagingAttacker>, Without<OgreChargeIndicator>),
    >,
    mut charge_targets: Query<
        (Entity, &Transform, &Hitbox, &Team, &mut Health, Option<&mut TemporaryHitPoints>),
        (Without<Boss>, Without<Corpse>, Without<BanishedModifier>, Without<OgreChargeIndicator>),
    >,
    mut indicator_query: Query<
        &mut Transform,
        (With<OgreChargeIndicator>, Without<Boss>),
    >,
) {
    let delta = time.delta_secs();

    for (
        _boss_entity,
        mut boss_transform,
        boss_team,
        mut charge_state,
        (rooted, sleeping, sleepwalking, banished, sickened, frozen, stunned),
    ) in &mut bosses
    {
        match charge_state.as_mut() {
            OgreChargeState::Idle { cooldown } => {
                *cooldown -= delta;
                if *cooldown <= 0.0 {
                    *charge_state = OgreChargeState::Targeting;
                }
            }

            OgreChargeState::Targeting => {
                let boss_pos = boss_transform.translation;

                let mut best_target: Option<(Vec3, f32)> = None;
                for (_entity, target_transform, target_team) in &potential_targets {
                    if !boss_team.is_enemy(target_team) {
                        continue;
                    }
                    let dx = target_transform.translation.x - boss_pos.x;
                    let dz = target_transform.translation.z - boss_pos.z;
                    let distance = (dx * dx + dz * dz).sqrt();

                    if distance < OGRE_CHARGE_TARGET_MIN_DISTANCE
                        || distance > OGRE_CHARGE_TARGET_MAX_DISTANCE
                    {
                        continue;
                    }

                    if best_target.map_or(true, |(_, d)| distance < d) {
                        best_target = Some((target_transform.translation, distance));
                    }
                }

                if let Some((target_pos, _)) = best_target {
                    let direction = Vec3::new(
                        target_pos.x - boss_pos.x,
                        0.0,
                        target_pos.z - boss_pos.z,
                    )
                    .normalize_or_zero();

                    let charge_distance = OGRE_CHARGE_MAX_DISTANCE;
                    let rotation = indicator_rotation(direction);
                    let perp = Vec3::new(-direction.z, 0.0, direction.x);
                    let lane_center = Vec3::new(
                        boss_pos.x + direction.x * (charge_distance / 2.0),
                        OGRE_CHARGE_INDICATOR_Y,
                        boss_pos.z + direction.z * (charge_distance / 2.0),
                    );
                    let half_width = OGRE_CHARGE_LANE_WIDTH / 2.0;

                    let mesh = ogre_assets.charge_rect_mesh.clone();
                    let mat = ogre_assets.charge_line_material.clone();

                    // Spawn outline: left, right, near (at ogre), far (at end)
                    let spawn_line = |cmds: &mut Commands, pos: Vec3, sx: f32, sy: f32| {
                        cmds.spawn((
                            Mesh3d(mesh.clone()),
                            MeshMaterial3d(mat.clone()),
                            Transform::from_translation(pos)
                                .with_rotation(rotation)
                                .with_scale(Vec3::new(sx, sy, 1.0)),
                            OgreChargeIndicator,
                            OnGameplayScreen,
                        )).id()
                    };
                    let t = OGRE_CHARGE_LINE_THICKNESS;
                    let left = spawn_line(&mut commands, lane_center + perp * half_width, t, charge_distance);
                    let right = spawn_line(&mut commands, lane_center - perp * half_width, t, charge_distance);
                    let near = spawn_line(&mut commands, boss_pos.with_y(OGRE_CHARGE_INDICATOR_Y), OGRE_CHARGE_LANE_WIDTH, t);
                    let far_pos = boss_pos + direction * charge_distance;
                    let far = spawn_line(&mut commands, far_pos.with_y(OGRE_CHARGE_INDICATOR_Y), OGRE_CHARGE_LANE_WIDTH, t);

                    // Unique emissive material for the fill — pulsed each frame
                    let fill_material = materials.add(StandardMaterial {
                        base_color: OGRE_CHARGE_FILL_BASE_COLOR,
                        emissive: bevy::color::LinearRgba::new(0.0, 0.0, 0.0, 1.0),
                        alpha_mode: AlphaMode::Blend,
                        unlit: false,
                        ..default()
                    });

                    let fill = commands
                        .spawn((
                            Mesh3d(ogre_assets.charge_rect_mesh.clone()),
                            MeshMaterial3d(fill_material.clone()),
                            Transform::from_translation(boss_pos.with_y(OGRE_CHARGE_INDICATOR_Y))
                                .with_rotation(rotation)
                                .with_scale(Vec3::new(OGRE_CHARGE_LANE_WIDTH, 1.0, 1.0)),
                            OgreChargeIndicator,
                            OnGameplayScreen,
                        ))
                        .id();

                    *charge_state = OgreChargeState::Telegraphing {
                        elapsed: 0.0,
                        direction,
                        target_distance: charge_distance,
                        indicators: ChargeIndicators { left, right, near, far, fill, fill_material },
                    };
                } else {
                    *charge_state = OgreChargeState::Idle { cooldown: 2.0 };
                }
            }

            OgreChargeState::Telegraphing {
                elapsed,
                direction,
                target_distance,
                indicators,
            } => {
                if crate::game::units::systems::is_cc_immobilized(
                    rooted,
                    sleeping,
                    sleepwalking,
                    banished,
                    sickened,
                    frozen,
                    stunned,
                ) {
                    despawn_indicators(&mut commands, &indicators.all());
                    *charge_state = OgreChargeState::Idle {
                        cooldown: OGRE_CHARGE_COOLDOWN / 2.0,
                    };
                    continue;
                }

                *elapsed += delta;
                let progress = (*elapsed / OGRE_CHARGE_TELEGRAPH_DURATION).min(1.0);
                let current_length = *target_distance * progress;

                let boss_pos = boss_transform.translation;
                let dir = *direction;

                // Grow the fill rectangle outward from the ogre
                if let Ok(mut fill_transform) = indicator_query.get_mut(indicators.fill) {
                    fill_transform.scale.y = current_length.max(1.0);

                    let half_length = current_length / 2.0;
                    fill_transform.translation = Vec3::new(
                        boss_pos.x + dir.x * half_length,
                        OGRE_CHARGE_INDICATOR_Y,
                        boss_pos.z + dir.z * half_length,
                    );
                }

                // Emissive glow: ramps up with progress, pulses ominously on top
                if let Some(mat) = materials.get_mut(&indicators.fill_material) {
                    let pulse = (*elapsed * OGRE_CHARGE_PULSE_FREQUENCY * std::f32::consts::TAU)
                        .sin()
                        * 0.5
                        + 0.5; // 0..1 oscillation
                    let intensity = progress * OGRE_CHARGE_EMISSIVE_MAX * (0.6 + 0.4 * pulse);
                    mat.emissive = bevy::color::LinearRgba::new(
                        intensity,
                        intensity * 0.08,
                        intensity * 0.02,
                        1.0,
                    );
                    // Fade base color in with progress
                    let alpha = 0.1 + progress * 0.4;
                    mat.base_color = Color::srgba(0.6, 0.05, 0.02, alpha);
                }

                if *elapsed >= OGRE_CHARGE_TELEGRAPH_DURATION {
                    despawn_indicators(&mut commands, &indicators.all());
                    *charge_state = OgreChargeState::Charging {
                        direction: dir,
                        distance_traveled: 0.0,
                        max_distance: *target_distance,
                        hit_entities: HashSet::new(),
                    };
                }
            }

            OgreChargeState::Charging {
                direction,
                distance_traveled,
                max_distance,
                hit_entities,
            } => {
                let move_delta = OGRE_CHARGE_SPEED * delta;
                boss_transform.translation.x += direction.x * move_delta;
                boss_transform.translation.z += direction.z * move_delta;
                *distance_traveled += move_delta;

                let boss_pos = boss_transform.translation;
                let dir = *direction;

                for (entity, target_transform, target_hitbox, target_team, mut health, temp_hp) in
                    &mut charge_targets
                {
                    if !boss_team.is_enemy(target_team) {
                        continue;
                    }
                    if hit_entities.contains(&entity) {
                        continue;
                    }

                    let dx = target_transform.translation.x - boss_pos.x;
                    let dz = target_transform.translation.z - boss_pos.z;
                    let distance = (dx * dx + dz * dz).sqrt();
                    let hit_range = OGRE_RADIUS + target_hitbox.radius + OGRE_CHARGE_HIT_EXTRA;

                    if distance > hit_range {
                        continue;
                    }

                    apply_damage_to_unit(&mut health, temp_hp.map(|t| t.into_inner()), OGRE_CHARGE_DAMAGE);
                    hit_entities.insert(entity);

                    // Knock units away perpendicular to the charge direction
                    let perp = Vec3::new(-dir.z, 0.0, dir.x);
                    let to_unit = Vec3::new(dx, 0.0, dz).normalize_or_zero();
                    let side = to_unit.dot(perp);
                    let knockback_dir = if side.abs() > 0.01 {
                        perp * side.signum()
                    } else {
                        perp
                    };

                    commands.entity(entity).insert(Knockback::new(
                        knockback_dir,
                        OGRE_CHARGE_KNOCKBACK_SPEED,
                        OGRE_CHARGE_KNOCKBACK_DURATION,
                    ));
                }

                if *distance_traveled >= *max_distance {
                    *charge_state = OgreChargeState::Recovery {
                        timer: OGRE_CHARGE_RECOVERY_DURATION,
                    };
                }
            }

            OgreChargeState::Recovery { timer } => {
                *timer -= delta;
                if *timer <= 0.0 {
                    *charge_state = OgreChargeState::Idle {
                        cooldown: OGRE_CHARGE_COOLDOWN,
                    };
                }
            }
        }
    }
}

/// Ogre rock throw — picks a target enemy within range and throws a rock at them.
/// Skipped during charge phases.
#[allow(clippy::type_complexity)]
pub fn ogre_rock_throw(
    time: Res<Time>,
    mut rock_events: MessageWriter<BoulderThrownMessage>,
    mut bosses: Query<
        (
            &Transform,
            &Team,
            &OgreChargeState,
            &mut RockThrowCooldown,
            (
                Option<&RootedModifier>,
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&crate::game::units::components::Stunned>,
                Option<&PolymorphedModifier>,
            ),
        ),
        (With<Boss>, Without<Corpse>),
    >,
    targets: Query<
        (&Transform, &Team),
        (Without<Corpse>, Without<BanishedModifier>, Without<StagingAttacker>),
    >,
) {
    let delta = time.delta_secs();

    for (
        boss_transform,
        boss_team,
        charge_state,
        mut cooldown,
        (rooted, sleeping, sleepwalking, banished, sickened, frozen, stunned, polymorphed),
    ) in &mut bosses
    {
        if charge_state.is_movement_locked() {
            continue;
        }
        if crate::game::units::systems::is_cc_immobilized(
            rooted, sleeping, sleepwalking, banished, sickened, frozen, stunned,
        ) || polymorphed.is_some()
        {
            continue;
        }

        cooldown.tick(delta);
        if !cooldown.is_ready() {
            continue;
        }

        if let Some(target_pos) = crate::game::units::systems::find_closest_enemy_in_range(
            boss_transform.translation,
            boss_team,
            ROCK_THROW_RANGE,
            &targets,
        ) {
            rock_events.write(BoulderThrownMessage {
                origin: boss_transform.translation,
                target: target_pos,
            });
            cooldown.reset(ROCK_THROW_COOLDOWN);
        }
    }
}

use crate::game::units::boss::utils::{indicator_rotation, despawn_indicators};
