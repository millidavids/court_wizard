use std::collections::HashSet;

use bevy::math::Affine2;
use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use super::resources::OgreAssets;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::resources::PathfindingGrid;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity, StagingAttacker};
use crate::game::terrain::boulder::constants::{ROCK_THROW_COOLDOWN, ROCK_THROW_RANGE};
use crate::game::terrain::boulder::messages::BoulderThrownMessage;
use crate::game::units::boss::components::Boss;
use crate::game::units::boss::lich::Lich;
use crate::game::units::brute::components::RockThrowCooldown;
use crate::game::units::components::{
    AttackTiming, BanishedModifier, CommanderAuraSpeedModifier, Corpse, DamageMultiplier,
    Effectiveness, EliteSpeedBonus, FlockingModifier, FlockingVelocity, FrozenSolidModifier,
    HasteModifier, Health, Hitbox, InMelee, Knockback, MovementSpeed, OriginalMaterial,
    PolymorphedModifier, RootedModifier, RoughTerrainModifier, SickenedModifier, SleepModifier,
    Sleepwalking, SlowMovementModifier, TargetingVelocity, Team, Teleportable, TemporaryHitPoints,
    apply_damage_to_unit,
};
use crate::game::units::components::{CombatAnimation, FacingDirection, WalkingAnimation};
use crate::game::units::random_position_in_cell;

/// Spawns the ogre at one of the tunnel spawn points.
pub fn spawn_ogre(
    rng: &mut impl Rng,
    mut commands: Commands,
    ogre_assets: Res<OgreAssets>,
    materials: &mut Assets<StandardMaterial>,
) {
    let (spawn_x, spawn_z) = attacker_spawn_position(0, 0.0);
    let (final_x, final_z) = random_position_in_cell(rng, spawn_x, spawn_z);

    let hitbox = Hitbox::new(OGRE_RADIUS, OGRE_HITBOX_HEIGHT);
    let spawn_y = OGRE_SPRITE_HEIGHT / 2.0 - OGRE_SPRITE_Y_OFFSET;

    // Initial velocity toward castle
    let to_center = Vec3::new(
        WIZARD_POSITION.x - final_x,
        0.0,
        WIZARD_POSITION.z - final_z,
    );
    let initial_velocity = to_center.normalize_or_zero() * OGRE_MOVEMENT_SPEED;

    let anim = WalkingAnimation {
        current_frame: 0,
        elapsed: rng.random::<f32>() * 0.125,
        columns: OGRE_SPRITE_COLUMNS,
        frame_uv: OGRE_FRAME_UV,
        direction_rows: OGRE_WALKING_DIRECTION_ROWS,
    };
    let material = crate::game::units::systems::create_sprite_material(
        materials,
        ogre_assets.walking_texture.clone(),
        OGRE_COLOR,
        OGRE_FRAME_UV,
        anim.uv_offset(FacingDirection::default()),
    );

    commands
        .spawn((
            // Rendering
            Mesh3d(ogre_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
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
        .insert((
            anim,
            FacingDirection::default(),
            crate::game::units::components::MeleeRangeBonus(OGRE_MELEE_RANGE_BONUS),
        ))
        .insert(RockThrowCooldown::new(8.0));
}

/// Overrides the ogre's facing direction to strongly prefer forward/backward.
/// Runs after the shared `update_facing_direction` to correct left/right
/// picks when the ogre is moving at a slight angle.
///
/// Filters on `With<OgreEnrageState>` (an ogre-only marker) — `With<Boss>`
/// would also match hags / dark mage / ray, which have their own facing logic
/// and would otherwise have their hysteresis-buffered facing clobbered each
/// frame by this raw-velocity override.
pub fn update_ogre_facing(
    camera_query: Query<&Transform, (With<Camera3d>, Without<Boss>)>,
    mut bosses: Query<
        (
            &Velocity,
            &mut FacingDirection,
            &WalkingAnimation,
            &MeshMaterial3d<StandardMaterial>,
        ),
        (
            With<OgreEnrageState>,
            Without<Corpse>,
            Without<CombatAnimation>,
        ),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let cam_forward = camera_transform.forward();
    let cam_forward_xz = Vec3::new(cam_forward.x, 0.0, cam_forward.z).normalize_or_zero();
    let cam_right = Vec3::new(-cam_forward_xz.z, 0.0, cam_forward_xz.x);

    for (velocity, mut facing, anim, material_handle) in &mut bosses {
        let vel_xz = Vec3::new(velocity.x, 0.0, velocity.z);
        if vel_xz.length_squared() < crate::game::units::components::ANIMATION_MOVE_THRESHOLD_SQ {
            continue;
        }

        let forward_dot = vel_xz.dot(cam_forward_xz);
        let right_dot = vel_xz.dot(cam_right);

        // Strong forward/back bias: only use left/right if the lateral component
        // is more than 3x the forward component
        let new_facing = if right_dot.abs() > forward_dot.abs() * 3.0 {
            if right_dot > 0.0 {
                FacingDirection::Right
            } else {
                FacingDirection::Left
            }
        } else if forward_dot < 0.0 {
            FacingDirection::Back
        } else {
            FacingDirection::Forward
        };

        if *facing != new_facing {
            *facing = new_facing;
            if let Some(mat) = materials.get_mut(material_handle) {
                mat.uv_transform = anim.uv_transform(new_facing);
            }
        }
    }
}

/// Updates ogre targeting velocity toward nearest enemy.
pub fn update_ogre_targeting(
    mut commands: Commands,
    mut bosses: Query<
        (Entity, &Transform, &Team, &mut TargetingVelocity),
        (With<Boss>, Without<Lich>),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Boss>,
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
        ),
    >,
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
#[allow(clippy::type_complexity)]
pub fn ogre_combat(
    time: Res<Time>,
    mut commands: Commands,
    ogre_assets: Res<OgreAssets>,
    game_config: Res<crate::config::GameConfig>,
    mut bosses: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut OgreAttackCooldown,
            &OgreChargeState,
        ),
        (
            With<Boss>,
            Without<Corpse>,
            Without<CombatAnimation>,
            Without<OgreThrowWindup>,
        ),
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

    for (boss_entity, boss_transform, boss_hitbox, boss_team, mut attack_cooldown, charge_state) in
        &mut bosses
    {
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

        // Play swing sound effect
        crate::game::units::wizard::spells::audio::play_sfx_scaled(
            &mut commands,
            &ogre_assets.swing_sfx,
            boss_pos,
            &game_config,
            1.0,
        );

        // Trigger attack animation
        commands.entity(boss_entity).insert(ogre_combat_animation(
            OGRE_ATTACKING_DIRECTION_ROWS,
            ogre_assets.attacking_texture.clone(),
            ogre_assets.walking_texture.clone(),
        ));

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
                Option<&crate::game::units::components::Petrified>,
            ),
        ),
        With<Boss>,
    >,
) {
    for (
        mut velocity,
        mut acceleration,
        movement_speed,
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
        (sleeping, sleepwalking, banished, polymorphed, sickened, frozen, stunned, petrified),
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
        if crate::game::units::systems::is_cc_immobilized(
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
            petrified,
        ) {
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
/// Modifies the sprite material's base_color to match the current enrage phase.
#[allow(clippy::type_complexity)]
pub fn update_enrage_state(
    mut bosses: Query<
        (
            &Health,
            &mut OgreEnrageState,
            &mut DamageMultiplier,
            &MeshMaterial3d<StandardMaterial>,
            Option<&OriginalMaterial>,
        ),
        With<Boss>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (health, mut enrage, mut damage_mult, mesh_material, original_material) in &mut bosses {
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

            let phase_tint = enrage_phase_tint(new_phase);

            // Update base_color on the per-entity sprite material.
            // If OriginalMaterial is present (spell effect active), update that
            // so the correct enrage tint restores when the effect ends.
            if let Some(orig) = original_material {
                if let Some(orig_mat) = materials.get_mut(&orig.0) {
                    orig_mat.base_color = phase_tint;
                }
            } else if let Some(mat) = materials.get_mut(&mesh_material.0) {
                mat.base_color = phase_tint;
            }
        }
    }
}

/// Returns the sprite tint color for a given enrage phase.
pub(super) fn enrage_phase_tint(phase: u8) -> Color {
    match phase {
        1 => OGRE_ENRAGE_1_COLOR,
        2 => OGRE_ENRAGE_2_COLOR,
        3 => OGRE_ENRAGE_3_COLOR,
        _ => OGRE_COLOR,
    }
}

/// Ogre charge ability — state machine handling targeting, telegraph, dash, and recovery.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn ogre_charge_system(
    time: Res<Time>,
    mut commands: Commands,
    ogre_assets: Res<OgreAssets>,
    game_config: Res<crate::config::GameConfig>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    pathfinding: Res<PathfindingGrid>,
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
                Option<&crate::game::units::components::Petrified>,
            ),
        ),
        (With<Boss>, Without<Corpse>),
    >,
    potential_targets: Query<
        (Entity, &Transform, &Team),
        (
            Without<Boss>,
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
            Without<OgreChargeIndicator>,
        ),
    >,
    mut charge_targets: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (
            Without<Boss>,
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<OgreChargeIndicator>,
        ),
    >,
    mut indicator_query: Query<&mut Transform, (With<OgreChargeIndicator>, Without<Boss>)>,
) {
    let delta = time.delta_secs();

    for (
        _boss_entity,
        mut boss_transform,
        boss_team,
        mut charge_state,
        (rooted, sleeping, sleepwalking, banished, sickened, frozen, stunned, petrified),
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

                    if !(OGRE_CHARGE_TARGET_MIN_DISTANCE..=OGRE_CHARGE_TARGET_MAX_DISTANCE)
                        .contains(&distance)
                    {
                        continue;
                    }

                    if best_target.is_none_or(|(_, d)| distance < d) {
                        best_target = Some((target_transform.translation, distance));
                    }
                }

                if let Some((target_pos, _)) = best_target {
                    let direction =
                        Vec3::new(target_pos.x - boss_pos.x, 0.0, target_pos.z - boss_pos.z)
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
                        ))
                        .id()
                    };
                    let t = OGRE_CHARGE_LINE_THICKNESS;
                    let left = spawn_line(
                        &mut commands,
                        lane_center + perp * half_width,
                        t,
                        charge_distance,
                    );
                    let right = spawn_line(
                        &mut commands,
                        lane_center - perp * half_width,
                        t,
                        charge_distance,
                    );
                    let near = spawn_line(
                        &mut commands,
                        boss_pos.with_y(OGRE_CHARGE_INDICATOR_Y),
                        OGRE_CHARGE_LANE_WIDTH,
                        t,
                    );
                    let far_pos = boss_pos + direction * charge_distance;
                    let far = spawn_line(
                        &mut commands,
                        far_pos.with_y(OGRE_CHARGE_INDICATOR_Y),
                        OGRE_CHARGE_LANE_WIDTH,
                        t,
                    );

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
                        indicators: ChargeIndicators {
                            left,
                            right,
                            near,
                            far,
                            fill,
                            fill_material,
                        },
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
                    petrified,
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

                    // Play charge sound effect
                    crate::game::units::wizard::spells::audio::play_sfx_scaled(
                        &mut commands,
                        &ogre_assets.charge_sfx,
                        boss_transform.translation,
                        &game_config,
                        1.0,
                    );

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
                let dir = *direction;

                // Check if the next position hits an obstacle (boulder, tree, wall)
                let next_pos = Vec3::new(
                    boss_transform.translation.x + dir.x * move_delta,
                    boss_transform.translation.y,
                    boss_transform.translation.z + dir.z * move_delta,
                );
                if pathfinding.sample_base_cost(next_pos) == f32::INFINITY {
                    *charge_state = OgreChargeState::Recovery {
                        timer: OGRE_CHARGE_RECOVERY_DURATION,
                    };
                    continue;
                }

                boss_transform.translation.x += dir.x * move_delta;
                boss_transform.translation.z += dir.z * move_delta;
                *distance_traveled += move_delta;

                let boss_pos = boss_transform.translation;

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

                    apply_damage_to_unit(
                        &mut health,
                        temp_hp.map(|t| t.into_inner()),
                        OGRE_CHARGE_DAMAGE,
                    );
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

/// Updates ogre sprite visuals during charge attack phases.
/// Swaps to attacking texture, sets the correct frame, applies red flash and vibration.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_ogre_charge_visuals(
    time: Res<Time>,
    ogre_assets: Res<OgreAssets>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Boss>)>,
    mut bosses: Query<
        (
            Entity,
            &mut Transform,
            &OgreChargeState,
            &OgreEnrageState,
            &MeshMaterial3d<StandardMaterial>,
            &WalkingAnimation,
            &mut FacingDirection,
            Option<&mut OgreChargeVisuals>,
        ),
        (With<Boss>, Without<Corpse>, Without<Camera3d>),
    >,
) {
    let cam_forward_xz = camera_query
        .single()
        .ok()
        .map(|cam| {
            let fwd = cam.forward();
            Vec3::new(fwd.x, 0.0, fwd.z).normalize_or_zero()
        })
        .unwrap_or(Vec3::NEG_Z);

    let delta = time.delta_secs();

    for (
        entity,
        mut transform,
        charge_state,
        enrage_state,
        material_handle,
        walking_anim,
        mut facing,
        charge_visuals,
    ) in &mut bosses
    {
        match charge_state {
            OgreChargeState::Telegraphing {
                elapsed, direction, ..
            } => {
                // Set facing direction from charge direction
                let new_facing = facing_from_world_direction(*direction, cam_forward_xz);
                *facing = new_facing;

                if let Some(mut visuals) = charge_visuals {
                    // Ongoing telegraph — update effects
                    visuals.elapsed += delta;
                    let progress = (*elapsed / OGRE_CHARGE_TELEGRAPH_DURATION).min(1.0);

                    if let Some(mat) = materials.get_mut(&material_handle.0) {
                        // First frame: swap texture to attacking sheet
                        if !visuals.texture_swapped {
                            mat.base_color_texture = Some(ogre_assets.attacking_texture.clone());
                            visuals.texture_swapped = true;
                        }

                        // Show frame 0 (wind-up) in correct direction
                        let row = OGRE_ATTACKING_DIRECTION_ROWS[new_facing as usize];
                        mat.uv_transform = ogre_frame_uv_transform(0, row);

                        // Red flash: pulse between enrage tint and flash color
                        let flash_t =
                            (visuals.elapsed * OGRE_CHARGE_FLASH_FREQUENCY * std::f32::consts::TAU)
                                .sin()
                                * 0.5
                                + 0.5;
                        let base_tint = enrage_phase_tint(enrage_state.phase);
                        mat.base_color = Color::LinearRgba(
                            base_tint
                                .to_linear()
                                .mix(&OGRE_CHARGE_FLASH_COLOR.to_linear(), flash_t),
                        );
                    }

                    // Vibration: sinusoidal offset scaled by progress
                    let amp = OGRE_CHARGE_VIBRATION_AMPLITUDE * progress;
                    let vib_x =
                        (visuals.elapsed * OGRE_CHARGE_VIBRATION_FREQ_X * std::f32::consts::TAU)
                            .sin()
                            * amp;
                    let vib_z =
                        (visuals.elapsed * OGRE_CHARGE_VIBRATION_FREQ_Z * std::f32::consts::TAU)
                            .sin()
                            * amp;
                    transform.translation.x = visuals.base_position.x + vib_x;
                    transform.translation.z = visuals.base_position.z + vib_z;
                } else {
                    // First frame of telegraph — insert visuals component
                    // and remove any active combat/throw animations
                    commands
                        .entity(entity)
                        .remove::<CombatAnimation>()
                        .remove::<OgreThrowWindup>()
                        .insert(OgreChargeVisuals {
                            texture_swapped: false,
                            elapsed: 0.0,
                            base_position: transform.translation,
                        });
                }
            }

            OgreChargeState::Charging { direction, .. } => {
                if let Some(mut visuals) = charge_visuals
                    && let Some(mat) = materials.get_mut(&material_handle.0)
                {
                    // Restore base position on first charging frame
                    // (remove vibration offset before charge movement begins)
                    if visuals.elapsed > 0.0 {
                        transform.translation.x = visuals.base_position.x;
                        transform.translation.z = visuals.base_position.z;
                        visuals.elapsed = 0.0;
                    }

                    // Show frame 1 (charge pose)
                    let new_facing = facing_from_world_direction(*direction, cam_forward_xz);
                    *facing = new_facing;
                    let row = OGRE_ATTACKING_DIRECTION_ROWS[new_facing as usize];
                    mat.uv_transform = ogre_frame_uv_transform(1, row);

                    // Restore normal tint (stop red flash)
                    mat.base_color = enrage_phase_tint(enrage_state.phase);
                }
            }

            OgreChargeState::Recovery { .. } => {
                if charge_visuals.is_some()
                    && let Some(mat) = materials.get_mut(&material_handle.0)
                {
                    let row = OGRE_ATTACKING_DIRECTION_ROWS[*facing as usize];
                    mat.uv_transform = ogre_frame_uv_transform(2, row);
                }
            }

            OgreChargeState::Idle { .. } | OgreChargeState::Targeting => {
                // Cleanup: restore walking texture and remove visuals
                if let Some(visuals) = charge_visuals {
                    if visuals.texture_swapped
                        && let Some(mat) = materials.get_mut(&material_handle.0)
                    {
                        mat.base_color_texture = Some(ogre_assets.walking_texture.clone());
                        mat.base_color = enrage_phase_tint(enrage_state.phase);
                        // Reset UV to walking idle frame
                        mat.uv_transform = walking_anim.uv_transform(*facing);
                    }
                    // Only restore base position if vibration was still active
                    // (CC interruption during telegraph). After charging starts,
                    // elapsed is reset to 0 and the ogre has moved legitimately.
                    if visuals.elapsed > 0.0 {
                        transform.translation.x = visuals.base_position.x;
                        transform.translation.z = visuals.base_position.z;
                    }
                    commands.entity(entity).remove::<OgreChargeVisuals>();
                }
            }
        }
    }
}

/// Creates a CombatAnimation configured for the ogre's sprite sheet dimensions.
fn ogre_combat_animation(
    direction_rows: [usize; 4],
    combat_texture: Handle<Image>,
    walking_texture: Handle<Image>,
) -> CombatAnimation {
    CombatAnimation {
        current_frame: 0,
        elapsed: 0.0,
        columns: OGRE_SPRITE_COLUMNS,
        frame_uv: OGRE_FRAME_UV,
        direction_rows,
        combat_texture,
        walking_texture,
        started: false,
    }
}

/// Returns the UV transform for a specific frame and direction row in the ogre sprite sheet.
fn ogre_frame_uv_transform(frame: usize, direction_row: usize) -> Affine2 {
    let uv_offset = Vec2::new(
        frame as f32 * OGRE_FRAME_UV.x,
        direction_row as f32 * OGRE_FRAME_UV.y,
    );
    Affine2::from_scale_angle_translation(OGRE_FRAME_UV, 0.0, uv_offset)
}

/// Derives a FacingDirection from a world-space direction vector relative to the camera.
fn facing_from_world_direction(dir: Vec3, cam_forward_xz: Vec3) -> FacingDirection {
    let cam_right = Vec3::new(-cam_forward_xz.z, 0.0, cam_forward_xz.x);
    let forward_dot = dir.dot(cam_forward_xz);
    let right_dot = dir.dot(cam_right);
    if forward_dot.abs() > right_dot.abs() {
        if forward_dot < 0.0 {
            FacingDirection::Back
        } else {
            FacingDirection::Forward
        }
    } else if right_dot > 0.0 {
        FacingDirection::Right
    } else {
        FacingDirection::Left
    }
}

/// Ogre rock throw — picks a target enemy within range and starts the throwing animation.
/// The boulder is launched when the animation finishes (see `ogre_throw_release`).
/// Skipped during charge phases or if already winding up.
#[allow(clippy::type_complexity)]
pub fn ogre_rock_throw(
    time: Res<Time>,
    ogre_assets: Res<OgreAssets>,
    game_config: Res<crate::config::GameConfig>,
    mut commands: Commands,
    mut bosses: Query<
        (
            Entity,
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
                Option<&crate::game::units::components::Petrified>,
                Option<&PolymorphedModifier>,
            ),
        ),
        (
            With<Boss>,
            Without<Corpse>,
            Without<OgreThrowWindup>,
            Without<CombatAnimation>,
        ),
    >,
    targets: Query<
        (&Transform, &Team),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (
        entity,
        boss_transform,
        boss_team,
        charge_state,
        mut cooldown,
        (
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
            petrified,
            polymorphed,
        ),
    ) in &mut bosses
    {
        if charge_state.is_movement_locked() {
            continue;
        }
        if crate::game::units::systems::is_cc_immobilized(
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
            petrified,
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
            // Play grunt sound effect
            crate::game::units::wizard::spells::audio::play_sfx_scaled(
                &mut commands,
                &ogre_assets.grunt_sfx,
                boss_transform.translation,
                &game_config,
                1.0,
            );

            // Start throwing animation and store target for release
            commands.entity(entity).insert((
                OgreThrowWindup {
                    target: target_pos,
                    sprite_index: 1,
                },
                ogre_combat_animation(
                    OGRE_THROWING_DIRECTION_ROWS,
                    ogre_assets.throwing_texture.clone(),
                    ogre_assets.walking_texture.clone(),
                ),
            ));
            cooldown.reset(ROCK_THROW_COOLDOWN);
        }
    }
}

/// Fires the boulder when the throwing animation finishes.
/// Detects completion by checking for `OgreThrowWindup` without `CombatAnimation`
/// (the shared animation system removes `CombatAnimation` when it's done).
pub fn ogre_throw_release(
    mut commands: Commands,
    mut rock_events: MessageWriter<BoulderThrownMessage>,
    bosses: Query<
        (Entity, &Transform, &OgreThrowWindup),
        (With<Boss>, Without<CombatAnimation>, Without<Corpse>),
    >,
) {
    for (entity, boss_transform, windup) in &bosses {
        rock_events.write(BoulderThrownMessage {
            origin: boss_transform.translation,
            target: windup.target,
            sprite_index: windup.sprite_index,
        });
        commands.entity(entity).remove::<OgreThrowWindup>();
    }
}

use crate::game::units::boss::utils::{despawn_indicators, indicator_rotation};
