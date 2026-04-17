use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use super::resources::RayAssets;
use crate::config::GameConfig;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::seeded_rng::resources::GameRng;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::Flying;
use crate::game::units::components::apply_spell_damage;
use crate::game::units::components::{
    AttackTiming, BerserkerRageModifier, Corpse, DamageMultiplier, Effectiveness, FearModifier,
    FlockingModifier, FlockingVelocity, Health, Hitbox, MindControlled, MovementSpeed,
    OriginalMaterial, Petrified, TargetingVelocity, Team, Teleportable, TemporaryHitPoints,
};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::{King, SpellShield};
use crate::game::units::wizard::spells::audio::SpellSfxAssets;

/// Attenuated volume for Ray's sound effects — slight falloff from wizard/camera position.
fn ray_sfx_volume(effect_pos: Vec3, game_config: &GameConfig) -> f32 {
    const RAY_SFX_BASE_SCALE: f32 = 0.6;
    const RAY_SFX_MAX_DIST: f32 = 8000.0;
    let distance = effect_pos.distance(SPELL_ORIGIN);
    let linear = (1.0 - distance / RAY_SFX_MAX_DIST).clamp(0.0, 1.0);
    game_config.effective_sfx_volume() * linear * RAY_SFX_BASE_SCALE
}
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

// ===== Spawn =====

pub fn spawn_ray(rng: &mut impl Rng, mut commands: Commands, assets: Res<RayAssets>) {
    let (spawn_x, spawn_z) = attacker_spawn_position(0, 0.0);
    let (final_x, final_z) = crate::game::units::random_position_in_cell(rng, spawn_x, spawn_z);

    let hitbox = Hitbox::new(RAY_BODY_RADIUS, RAY_BODY_HITBOX_HEIGHT);
    let spawn_y = RAY_BODY_HOVER_HEIGHT + hitbox.height / 2.0;

    let to_center = Vec3::new(
        WIZARD_POSITION.x - final_x,
        0.0,
        WIZARD_POSITION.z - final_z,
    );
    let initial_velocity = to_center.normalize_or_zero() * RAY_APPROACH_SPEED;

    commands
        .spawn((
            Mesh3d(assets.body_mesh.clone()),
            MeshMaterial3d(assets.material_phase0.clone()),
            Transform::from_xyz(final_x, spawn_y, final_z),
            Velocity {
                x: initial_velocity.x,
                z: initial_velocity.z,
                ..default()
            },
            Acceleration::new(),
            hitbox,
            Health::new(RAY_HEALTH),
            MovementSpeed(RAY_APPROACH_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Attackers,
            Boss,
            Ray,
        ))
        .insert((
            RayState::Approaching,
            RayEyeState::new(),
            DamageMultiplier(RAY_DAMAGE_MULTIPLIER),
            crate::game::units::boss::ogre::MeleeDamageReduction {
                multiplier: RAY_MELEE_DAMAGE_REDUCTION,
            },
            TargetingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Attacker,
            FlockingVelocity::default(),
            FlockingModifier::new(0.0, 0.0, 0.0),
            Teleportable,
            Billboard,
            OnGameplayScreen,
        ))
        .insert((
            RayDisintegrationSweep {
                beam_entity: None,
                glow_entity: None,
                sfx_entity: None,
                tip_position: Vec2::ZERO,
                tip_velocity: Vec2::ZERO,
                cooldown: 0.0,
            },
            RayPetrificationSweep {
                beam_entity: None,
                glow_entity: None,
                cooldown: 3.0,
            },
            RayFearSweep {
                beam_entity: None,
                glow_entity: None,
                fear_cooldown: 0.0,
            },
            RayTeleportSweep { cooldown: 5.0 },
            RayMindControlSweep {
                beam_entity: None,
                glow_entity: None,
                cooldown: 4.0,
            },
        ));

    let angle_step = std::f32::consts::TAU / 6.0;
    for (i, &eye_type) in RayEyeType::ALL.iter().enumerate() {
        let angle = angle_step * i as f32;
        let offset_x = angle.cos() * RAY_EYE_WANDER_RADIUS * 0.5;
        let offset_z = angle.sin() * RAY_EYE_WANDER_RADIUS * 0.5;

        let eye_x = final_x + offset_x;
        let eye_z = final_z + offset_z;
        let eye_y = RAY_EYE_FLOAT_HEIGHT;

        commands.spawn((
            Mesh3d(assets.eye_mesh.clone()),
            MeshMaterial3d(assets.eye_materials[eye_type.index()].clone()),
            Transform::from_xyz(eye_x, eye_y, eye_z),
            Hitbox::new(RAY_EYE_RADIUS, RAY_EYE_HITBOX_HEIGHT),
            Health::new(RAY_EYE_HEALTH),
            Effectiveness::new(),
            AttackTiming::new(),
            Team::Attackers,
            RayEye {
                eye_type,
                heading: Vec2::new(angle.cos(), angle.sin()),
            },
            Flying,
            Billboard,
            OnGameplayScreen,
        ));
    }
}

// ===== Movement =====

#[allow(clippy::too_many_arguments)]
pub fn ray_movement(
    time: Res<Time>,
    mut bosses: Query<
        (
            &Transform,
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            &mut RayState,
        ),
        (With<Ray>, Without<Corpse>),
    >,
) {
    for (
        transform,
        mut velocity,
        mut acceleration,
        movement_speed,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        mut state,
    ) in &mut bosses
    {
        if matches!(*state, RayState::Approaching) {
            crate::game::units::systems::calculate_weighted_movement(
                &time,
                &mut velocity,
                &mut acceleration,
                movement_speed.0,
                targeting_velocity,
                flocking_velocity,
                flow_field_velocity,
                false,
                None,
                None,
                None,
                None,
                None,
                None,
            );
            if transform.translation.x <= RAY_APPROACH_TARGET_X {
                *state = RayState::Idle;
            }
            continue;
        }

        crate::game::units::systems::calculate_weighted_movement(
            &time,
            &mut velocity,
            &mut acceleration,
            RAY_HOVER_SPEED,
            targeting_velocity,
            flocking_velocity,
            flow_field_velocity,
            false,
            None,
            None,
            None,
            None,
            None,
            None,
        );
    }
}


// ===== Body/Eye Health Management =====

/// Ray's body is a sentinel — defeat is determined by all eyes dying, not body HP.
/// Any damage dealt to the body is redistributed evenly (1/5) across all eyes, dead
/// or alive, then the body is topped back up.
pub fn ray_body_damage_to_eyes(
    mut body_query: Query<&mut Health, (With<Ray>, Without<RayEye>, Without<Corpse>)>,
    mut eyes: Query<&mut Health, (With<RayEye>, Without<Ray>, Without<RayEyeDying>)>,
) {
    let Ok(mut body_health) = body_query.single_mut() else {
        return;
    };

    let damage_taken = body_health.max - body_health.current;
    if damage_taken <= 0.0 {
        return;
    }

    let per_eye = damage_taken / RayEyeType::COUNT as f32;
    for mut eye_health in &mut eyes {
        eye_health.current = (eye_health.current - per_eye).max(0.0);
    }

    body_health.current = body_health.max;
}

/// Detects eyes that have reached 0 HP, marks them inactive, and starts their implode animation.
pub fn ray_eye_death_check(
    mut commands: Commands,
    eyes: Query<(Entity, &Health, &RayEye, &Transform), (Without<RayEyeDying>, Without<Ray>)>,
    mut body_query: Query<&mut RayEyeState, With<Ray>>,
    sfx_assets: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    let Ok(mut eye_state) = body_query.single_mut() else {
        return;
    };
    for (entity, health, eye, transform) in &eyes {
        if health.current <= 0.0 {
            let idx = eye.eye_type.index();
            if eye_state.active[idx] {
                eye_state.active[idx] = false;
            }
            commands.entity(entity).insert(RayEyeDying {
                time_alive: 0.0,
                duration: RAY_EYE_IMPLODE_DURATION,
                initial_scale: transform.scale.x.max(1.0),
            });

            let volume = ray_sfx_volume(transform.translation, &game_config);
            if volume > 0.0 {
                commands.spawn((
                    bevy::audio::AudioPlayer::new(sfx_assets.ray_eye_death.clone()),
                    bevy::audio::PlaybackSettings::ONCE
                        .with_volume(bevy::audio::Volume::Linear(volume)),
                    OnGameplayScreen,
                ));
            }
        }
    }
}

/// Animates dying eyes imploding then despawns them.
pub fn update_ray_dying_eyes(
    time: Res<Time>,
    mut commands: Commands,
    mut eyes: Query<(Entity, &mut RayEyeDying, &mut Transform)>,
) {
    let delta = time.delta_secs();
    for (entity, mut dying, mut transform) in &mut eyes {
        dying.time_alive += delta;
        if dying.time_alive >= dying.duration {
            commands.entity(entity).try_despawn();
            continue;
        }
        let t = (dying.time_alive / dying.duration).clamp(0.0, 1.0);
        let scale = dying.initial_scale * (1.0 - t).max(0.0).powi(2);
        transform.scale = Vec3::splat(scale);
    }
}

/// Once all eyes have died (active = false for all), kill Ray's body — triggering victory.
pub fn ray_all_eyes_dead_check(
    mut commands: Commands,
    body_query: Query<(Entity, &RayEyeState), (With<Ray>, Without<Corpse>)>,
) {
    for (entity, eye_state) in &body_query {
        if eye_state.active.iter().all(|&a| !a) {
            commands.entity(entity).insert(Corpse);
        }
    }
}

pub fn ray_death_cleanup(
    mut commands: Commands,
    dead_ray: Query<Entity, (With<Ray>, With<Corpse>)>,
    eyes: Query<Entity, With<RayEye>>,
    particles: Query<Entity, With<RayStalkParticle>>,
    mut sweep_disintegrate: Query<&mut RayDisintegrationSweep>,
    mut sweep_petrify: Query<&mut RayPetrificationSweep>,
    mut sweep_fear: Query<&mut RayFearSweep>,
    mut sweep_mind_control: Query<&mut RayMindControlSweep>,
) {
    if dead_ray.iter().next().is_none() {
        return;
    }

    for entity in &eyes {
        commands.entity(entity).try_despawn();
    }
    for entity in &particles {
        commands.entity(entity).try_despawn();
    }
    for mut sweep in &mut sweep_disintegrate {
        despawn_ray_beam(&mut commands, &mut sweep);
    }
    for mut sweep in &mut sweep_petrify {
        despawn_petrify_beam(&mut commands, &mut sweep);
    }
    for mut sweep in &mut sweep_fear {
        despawn_fear_beam(&mut commands, &mut sweep);
    }
    for mut sweep in &mut sweep_mind_control {
        despawn_mind_control_beam(&mut commands, &mut sweep);
    }
}


// ===== Eye Movement =====

pub fn update_ray_eye_movement(
    time: Res<Time>,
    mut commands: Commands,
    body_query: Query<(&Transform, &RayEyeState), (With<Ray>, Without<RayEye>)>,
    mut eyes: Query<
        (
            Entity,
            &mut Transform,
            &mut RayEye,
            &MeshMaterial3d<StandardMaterial>,
            Has<OriginalMaterial>,
        ),
        Without<Ray>,
    >,
    ray_assets: Res<RayAssets>,
    mut game_rng: ResMut<GameRng>,
) {
    let delta = time.delta_secs();
    let Ok((body_transform, eye_state)) = body_query.single() else {
        return;
    };
    let body_pos = body_transform.translation;

    // Collect positions for eye-to-eye separation
    let eye_positions: Vec<(Entity, Vec2)> = eyes
        .iter()
        .map(|(e, tf, _, _, _)| (e, Vec2::new(tf.translation.x, tf.translation.z)))
        .collect();

    for (entity, mut transform, mut eye, material, has_original_material) in &mut eyes {
        let i = eye.eye_type.index();
        // Only swap active/inactive material when no spell effect is tinting us.
        // The OriginalMaterial component indicates a spell effect owns the material.
        if !has_original_material {
            let new_handle = if eye_state.active[i] {
                &ray_assets.eye_materials[i]
            } else {
                &ray_assets.eye_inactive_material
            };
            if material.0 != *new_handle {
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(new_handle.clone()));
            }
        }

        let my_pos = Vec2::new(transform.translation.x, transform.translation.z);

        // Fear eye: orbit Ray at a fixed radius instead of wandering
        if eye.eye_type == RayEyeType::Fear && eye_state.active[RayEyeType::Fear.index()] {
            // Use heading.x as orbit angle storage
            eye.heading.x += FEAR_EYE_ORBIT_SPEED * delta;
            let angle = eye.heading.x;
            let body_xz = Vec2::new(body_pos.x, body_pos.z);
            let orbit_pos = body_xz + Vec2::new(angle.cos(), angle.sin()) * FEAR_EYE_ORBIT_RADIUS;
            transform.translation.x = orbit_pos.x;
            transform.translation.z = orbit_pos.y;
            transform.translation.y = RAY_EYE_FLOAT_HEIGHT;
            continue;
        }

        // Separation force from other eyes
        let mut separation = Vec2::ZERO;
        for &(other_entity, other_pos) in &eye_positions {
            if other_entity == entity {
                continue;
            }
            let diff = my_pos - other_pos;
            let dist = diff.length();
            if dist > 0.001 && dist < RAY_EYE_SEPARATION_RADIUS {
                let push = (1.0 - dist / RAY_EYE_SEPARATION_RADIUS) * RAY_EYE_SEPARATION_FORCE;
                separation += (diff / dist) * push;
            }
        }

        // Steer away from Ray when close, toward Ray when far
        let to_body = Vec2::new(body_pos.x - my_pos.x, body_pos.z - my_pos.y);
        let dist_to_body = to_body.length();
        let orbit_threshold = RAY_EYE_RADIUS * 4.0;

        let target_dir = if dist_to_body < 0.001 {
            eye.heading
        } else if dist_to_body < orbit_threshold {
            // Too close — steer away
            -(to_body / dist_to_body)
        } else {
            // Too far — steer back
            to_body / dist_to_body
        };

        let current_dir = eye.heading.normalize_or_zero();

        let dot = current_dir.dot(target_dir).clamp(-1.0, 1.0);
        let angle_to_target = dot.acos();
        let max_turn = RAY_EYE_TURN_RATE * delta;

        let new_dir = if angle_to_target <= max_turn || current_dir == Vec2::ZERO {
            target_dir
        } else {
            let cross = current_dir.x * target_dir.y - current_dir.y * target_dir.x;
            let turn_sign = if cross >= 0.0 { 1.0 } else { -1.0 };
            let cos_t = max_turn.cos();
            let sin_t = max_turn.sin() * turn_sign;
            Vec2::new(
                current_dir.x * cos_t - current_dir.y * sin_t,
                current_dir.x * sin_t + current_dir.y * cos_t,
            )
            .normalize_or_zero()
        };

        // Random angular drift so eyes don't converge on the same path
        let rng = &mut game_rng.0;
        let drift = (rng.r#gen::<f32>() - 0.5) * RAY_EYE_DRIFT_RATE * delta;
        let cos_d = drift.cos();
        let sin_d = drift.sin();
        let drifted = Vec2::new(
            new_dir.x * cos_d - new_dir.y * sin_d,
            new_dir.x * sin_d + new_dir.y * cos_d,
        )
        .normalize_or_zero();

        eye.heading = drifted;

        // Move at constant speed + separation push
        transform.translation.x += (drifted.x * RAY_EYE_WANDER_SPEED + separation.x) * delta;
        transform.translation.z += (drifted.y * RAY_EYE_WANDER_SPEED + separation.y) * delta;

        // Pull back if too far from body
        if dist_to_body > RAY_EYE_WANDER_RADIUS {
            let pull = (dist_to_body - RAY_EYE_WANDER_RADIUS) * 2.0 * delta;
            let pull_dir = to_body / dist_to_body;
            transform.translation.x += pull_dir.x * pull;
            transform.translation.z += pull_dir.y * pull;
        }

        transform.translation.y = RAY_EYE_FLOAT_HEIGHT;
    }
}

// ===== Stalk Particles =====

pub fn spawn_ray_stalk_particles(
    time: Res<Time>,
    mut commands: Commands,
    body_query: Query<&Transform, (With<Ray>, Without<RayEye>)>,
    eyes: Query<(Entity, &Transform, &RayEye), Without<RayEyeDying>>,
    ray_assets: Res<RayAssets>,
    mut game_rng: ResMut<GameRng>,
    mut spawn_timer: Local<f32>,
) {
    let Ok(body_tf) = body_query.single() else {
        return;
    };

    *spawn_timer += time.delta_secs();
    if *spawn_timer < RAY_STALK_PARTICLE_SPAWN_INTERVAL {
        return;
    }
    *spawn_timer -= RAY_STALK_PARTICLE_SPAWN_INTERVAL;

    let body_pos = body_tf.translation;
    let rng = &mut game_rng.0;

    for (eye_entity, eye_tf, _) in &eyes {
        let to_eye = eye_tf.translation - body_pos;
        let dir = to_eye.normalize_or_zero();
        let wobble_offset = rng.r#gen::<f32>() * std::f32::consts::TAU;

        // Initial velocity toward the eye with some spread
        let spread_x = rng.r#gen::<f32>() * 0.3 - 0.15;
        let spread_z = rng.r#gen::<f32>() * 0.3 - 0.15;
        let initial_vel = (dir + Vec3::new(spread_x, 0.0, spread_z)).normalize_or_zero()
            * RAY_STALK_PARTICLE_SPEED;

        commands.spawn((
            Mesh3d(ray_assets.particle_mesh.clone()),
            MeshMaterial3d(ray_assets.particle_material.clone()),
            Transform::from_translation(body_pos),
            RayStalkParticle {
                eye_entity,
                velocity: initial_vel,
                time_alive: 0.0,
                wobble_offset,
            },
            Billboard,
            OnGameplayScreen,
        ));
    }
}

pub fn update_ray_stalk_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Transform, &mut RayStalkParticle)>,
    eye_transforms: Query<&Transform, (With<RayEye>, Without<RayStalkParticle>)>,
    mut game_rng: ResMut<GameRng>,
) {
    let delta = time.delta_secs();

    for (entity, mut transform, mut particle) in &mut particles {
        particle.time_alive += delta;

        let Ok(eye_tf) = eye_transforms.get(particle.eye_entity) else {
            commands.entity(entity).try_despawn();
            continue;
        };
        let eye_pos = eye_tf.translation;

        let to_eye = eye_pos - transform.translation;
        let dist = to_eye.length();

        if dist < RAY_STALK_PARTICLE_RADIUS * 2.0 || particle.time_alive >= 5.0 {
            commands.entity(entity).try_despawn();
            continue;
        }

        let desired = to_eye.normalize_or_zero() * RAY_STALK_PARTICLE_SPEED;
        let steer = (desired - particle.velocity) * (RAY_STALK_PARTICLE_HOMING * delta).min(1.0);
        particle.velocity += steer;

        let speed = particle.velocity.length();
        if speed > RAY_STALK_PARTICLE_SPEED {
            particle.velocity = particle.velocity.normalize() * RAY_STALK_PARTICLE_SPEED;
        }

        // Wobble perpendicular to travel direction
        let t = particle.time_alive * RAY_STALK_PARTICLE_WOBBLE_FREQ + particle.wobble_offset;
        let forward = particle.velocity.normalize_or_zero();
        let right = Vec3::new(-forward.z, 0.0, forward.x);
        let wobble = right * t.sin() * RAY_STALK_PARTICLE_WOBBLE_AMP
            + Vec3::Y * (t * 1.3).cos() * RAY_STALK_PARTICLE_WOBBLE_AMP * 0.5;

        // Random shudder
        let rng = &mut game_rng.0;
        let shudder = Vec3::new(
            (rng.r#gen::<f32>() - 0.5) * 2.0,
            (rng.r#gen::<f32>() - 0.5) * 2.0,
            (rng.r#gen::<f32>() - 0.5) * 2.0,
        ) * RAY_STALK_PARTICLE_SHUDDER;

        transform.translation += (particle.velocity + wobble + shudder) * delta;
    }
}

// ===== Beam Visual Cleanup =====

pub fn update_ray_beam_visuals(
    time: Res<Time>,
    mut commands: Commands,
    mut beams: Query<(Entity, &mut RayBeamVisual)>,
) {
    let delta = time.delta_secs();
    for (entity, mut beam) in &mut beams {
        beam.lifetime -= delta;
        if beam.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// ===== Fear Movement Override =====

pub fn cleanse_fear_with_rage(
    mut commands: Commands,
    rage_query: Query<Entity, (With<FearModifier>, With<BerserkerRageModifier>)>,
    mc_query: Query<Entity, (With<FearModifier>, With<MindControlled>)>,
) {
    for entity in &rage_query {
        commands.entity(entity).remove::<FearModifier>();
    }
    for entity in &mc_query {
        commands.entity(entity).remove::<FearModifier>();
    }
}

pub fn update_fear_movement(
    mut query: Query<(
        &Transform,
        &FearModifier,
        &mut Velocity,
        &mut Acceleration,
        &MovementSpeed,
    )>,
) {
    for (transform, fear, mut velocity, mut acceleration, movement_speed) in query.iter_mut() {
        let flee_dir = transform.translation - fear.flee_from;
        let horizontal = Vec3::new(flee_dir.x, 0.0, flee_dir.z);
        let length = horizontal.length();
        if length > 0.001 {
            let normalized = horizontal / length;
            let speed = movement_speed.0 * 1.5;
            velocity.x = normalized.x * speed;
            velocity.z = normalized.z * speed;
            acceleration.reset();
        }
    }
}

// ===== Disintegration Sweep =====

#[allow(clippy::too_many_arguments)]
pub fn ray_disintegration_sweep(
    time: Res<Time>,
    mut commands: Commands,
    mut bosses: Query<
        (&Transform, &RayEyeState, &mut RayDisintegrationSweep),
        (With<Ray>, Without<Corpse>, Without<RayEye>),
    >,
    eye_query: Query<(&Transform, &RayEye)>,
    mut defenders: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &mut Health,
            Option<&SpellShield>,
            Option<&mut TemporaryHitPoints>,
        ),
        (With<Team>, Without<Corpse>, Without<Boss>, Without<RayEye>),
    >,
    team_query: Query<&Team>,
    ray_assets: Res<RayAssets>,
    spell_assets: Res<SpellVisualAssets>,
    sfx_assets: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    mut beams: Query<&mut RayDisintegrateBeam>,
    mut game_rng: ResMut<GameRng>,
) {
    let delta = time.delta_secs();

    for (boss_transform, eye_state, mut sweep) in &mut bosses {
        let boss_pos = boss_transform.translation;

        if !eye_state.is_disintegration_active() {
            despawn_ray_beam(&mut commands, &mut sweep);
            continue;
        }

        // Cooldown between beam firings
        if sweep.beam_entity.is_none() && sweep.cooldown > 0.0 {
            sweep.cooldown -= delta;
            continue;
        }

        let has_targets = defenders.iter().any(|(entity, def_transform, _, _, _, _)| {
            if let Ok(team) = team_query.get(entity) {
                if *team != Team::Defenders {
                    return false;
                }
            }
            let horizontal = Vec2::new(
                def_transform.translation.x - boss_pos.x,
                def_transform.translation.z - boss_pos.z,
            );
            horizontal.length() <= MAX_BEAM_RANGE
        });

        if !has_targets {
            despawn_ray_beam(&mut commands, &mut sweep);
            continue;
        }

        let eye_pos = eye_query
            .iter()
            .find(|(_, eye)| eye.eye_type == RayEyeType::Disintegration)
            .map(|(tf, _)| tf.translation)
            .unwrap_or(boss_pos);

        // Spawn beam aiming at nearest target, with random velocity away from it
        if sweep.beam_entity.is_none() {
            let rng = &mut game_rng.0;

            // Find nearest defender position and place reticle directly on them
            let target_pos = find_nearest_defender_position(boss_pos, &defenders, &team_query);
            sweep.tip_position = if let Some(pos) = target_pos {
                Vec2::new(pos.x, pos.z)
            } else {
                Vec2::new(boss_pos.x, boss_pos.z)
            };

            // Random velocity direction carrying it away from the target
            let angle = rng.r#gen::<f32>() * std::f32::consts::TAU;
            sweep.tip_velocity = Vec2::new(angle.cos(), angle.sin()) * DISINTEGRATION_RETICLE_SPEED;

            let ground_target = Vec3::new(sweep.tip_position.x, 0.0, sweep.tip_position.y);
            let dir = (ground_target - eye_pos).normalize_or_zero();
            let length = (ground_target - eye_pos).length();
            let beam = RayDisintegrateBeam::new(eye_pos, dir, length);

            let beam_entity = commands
                .spawn((
                    beam,
                    Mesh3d(spell_assets.disintegrate_cone.clone()),
                    MeshMaterial3d(
                        ray_assets.beam_materials[RayEyeType::Disintegration.index()].clone(),
                    ),
                    Transform::from_translation(eye_pos),
                    OnGameplayScreen,
                ))
                .id();

            let glow_entity = commands
                .spawn((
                    RayDisintegrateGlow { beam_entity },
                    Mesh3d(spell_assets.disintegrate_cone.clone()),
                    MeshMaterial3d(
                        ray_assets.beam_materials[RayEyeType::Disintegration.index()].clone(),
                    ),
                    Transform::from_translation(eye_pos),
                    OnGameplayScreen,
                ))
                .id();

            sweep.beam_entity = Some(beam_entity);
            sweep.glow_entity = Some(glow_entity);
            let volume = ray_sfx_volume(eye_pos, &game_config);
            if volume > 0.0 {
                let sfx_id = commands
                    .spawn((
                        bevy::audio::AudioPlayer::new(sfx_assets.disintegrate_channel.clone()),
                        bevy::audio::PlaybackSettings::LOOP
                            .with_volume(bevy::audio::Volume::Linear(volume)),
                        OnGameplayScreen,
                    ))
                    .id();
                sweep.sfx_entity = Some(sfx_id);
            }
        }

        if let Some(beam_entity) = sweep.beam_entity {
            // Steer reticle velocity toward nearest defender (aerialist-style turn rate)
            let desired_dir =
                find_nearest_defender_direction_from(sweep.tip_position, &defenders, &team_query);
            if let Some(desired) = desired_dir {
                let current = sweep.tip_velocity.normalize_or_zero();
                let dot = current.dot(desired).clamp(-1.0, 1.0);
                let angle_to_target = dot.acos();
                let max_turn = DISINTEGRATION_TURN_RATE * delta;

                sweep.tip_velocity = if angle_to_target <= max_turn || current == Vec2::ZERO {
                    desired * DISINTEGRATION_RETICLE_SPEED
                } else {
                    let cross = current.x * desired.y - current.y * desired.x;
                    let turn_sign = if cross >= 0.0 { 1.0 } else { -1.0 };
                    let cos_t = max_turn.cos();
                    let sin_t = max_turn.sin() * turn_sign;
                    let new_dir = Vec2::new(
                        current.x * cos_t - current.y * sin_t,
                        current.x * sin_t + current.y * cos_t,
                    );
                    new_dir.normalize_or_zero() * DISINTEGRATION_RETICLE_SPEED
                };
            }

            // Move the reticle
            let vel = sweep.tip_velocity;
            sweep.tip_position += vel * delta;

            // Compute 3D beam from eye down to ground reticle
            let ground_target = Vec3::new(sweep.tip_position.x, 0.0, sweep.tip_position.y);
            let to_target = ground_target - eye_pos;
            let beam_length = to_target.length().max(1.0);
            let beam_dir = to_target / beam_length;

            if let Ok(mut beam) = beams.get_mut(beam_entity) {
                beam.origin = eye_pos;
                beam.direction = beam_dir;
                beam.length = beam_length;
                beam.time_alive += delta;

                if beam.time_alive >= DISINTEGRATION_BEAM_LIFETIME {
                    despawn_ray_beam(&mut commands, &mut sweep);
                    sweep.cooldown = DISINTEGRATION_BEAM_COOLDOWN;
                    continue;
                }

                // Cone-cylinder intersection damage
                beam.time_since_damage += delta;
                if beam.time_since_damage >= DISINTEGRATION_DAMAGE_INTERVAL {
                    beam.time_since_damage = 0.0;
                    let cone_base_radius = BEAM_WIDTH * 0.15 * 0.7;
                    let targets = find_units_in_cone(
                        eye_pos,
                        beam_dir,
                        BEAM_LENGTH,
                        cone_base_radius,
                        &defenders,
                        &team_query,
                    );
                    for &entity in &targets {
                        if let Ok((_, _, _, mut health, spell_shield, temp_hp)) =
                            defenders.get_mut(entity)
                        {
                            apply_spell_damage(
                                &mut commands,
                                entity,
                                &mut health,
                                temp_hp.map(|t| t.into_inner()),
                                DISINTEGRATION_DAMAGE_PER_TICK,
                                DamageType::Fire,
                                spell_shield.is_some(),
                            );
                        }
                    }
                }
            }
        }
    }
}

fn despawn_ray_beam(commands: &mut Commands, sweep: &mut RayDisintegrationSweep) {
    if let Some(entity) = sweep.beam_entity.take() {
        commands.entity(entity).try_despawn();
    }
    if let Some(entity) = sweep.glow_entity.take() {
        commands.entity(entity).try_despawn();
    }
    if let Some(entity) = sweep.sfx_entity.take() {
        commands.entity(entity).try_despawn();
    }
}

fn despawn_petrify_beam(commands: &mut Commands, sweep: &mut RayPetrificationSweep) {
    if let Some(entity) = sweep.beam_entity.take() {
        commands.entity(entity).try_despawn();
    }
    if let Some(entity) = sweep.glow_entity.take() {
        commands.entity(entity).try_despawn();
    }
}

// ===== Petrification Beam =====

#[allow(clippy::too_many_arguments)]
pub fn ray_petrification_beam(
    time: Res<Time>,
    mut commands: Commands,
    mut bosses: Query<
        (&Transform, &RayEyeState, &mut RayPetrificationSweep),
        (With<Ray>, Without<Corpse>, Without<RayEye>),
    >,
    eye_query: Query<(&Transform, &RayEye)>,
    defenders: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &mut Health,
            Option<&SpellShield>,
            Option<&mut TemporaryHitPoints>,
        ),
        (With<Team>, Without<Corpse>, Without<Boss>, Without<RayEye>),
    >,
    team_query: Query<&Team>,
    ray_assets: Res<RayAssets>,
    spell_assets: Res<SpellVisualAssets>,
    sfx_assets: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    mut beams: Query<&mut RayPetrificationBeam>,
) {
    let delta = time.delta_secs();

    for (boss_transform, eye_state, mut sweep) in &mut bosses {
        let boss_pos = boss_transform.translation;

        if !eye_state.active[RayEyeType::Petrification.index()] {
            despawn_petrify_beam(&mut commands, &mut sweep);
            continue;
        }

        if sweep.beam_entity.is_none() && sweep.cooldown > 0.0 {
            sweep.cooldown -= delta;
            continue;
        }

        let has_targets = defenders.iter().any(|(entity, def_transform, _, _, _, _)| {
            if let Ok(team) = team_query.get(entity) {
                if *team != Team::Defenders {
                    return false;
                }
            }
            let horizontal = Vec2::new(
                def_transform.translation.x - boss_pos.x,
                def_transform.translation.z - boss_pos.z,
            );
            horizontal.length() <= MAX_BEAM_RANGE
        });

        if !has_targets {
            despawn_petrify_beam(&mut commands, &mut sweep);
            continue;
        }

        let eye_pos = eye_query
            .iter()
            .find(|(_, eye)| eye.eye_type == RayEyeType::Petrification)
            .map(|(tf, _)| tf.translation)
            .unwrap_or(boss_pos);

        if sweep.beam_entity.is_none() {
            let target_pos = find_nearest_defender_position(boss_pos, &defenders, &team_query);
            let target = target_pos.unwrap_or(Vec3::new(boss_pos.x - 100.0, 0.0, boss_pos.z));
            let to_target = target - eye_pos;
            let dir = to_target.normalize_or_zero();
            let length = to_target.length().max(1.0);

            let beam = RayPetrificationBeam::new(eye_pos, dir, length);

            let beam_entity = commands
                .spawn((
                    beam,
                    Mesh3d(spell_assets.disintegrate_cone.clone()),
                    MeshMaterial3d(
                        ray_assets.beam_materials[RayEyeType::Petrification.index()].clone(),
                    ),
                    Transform::from_translation(eye_pos),
                    OnGameplayScreen,
                ))
                .id();

            let glow_entity = commands
                .spawn((
                    RayPetrificationGlow { beam_entity },
                    Mesh3d(spell_assets.disintegrate_cone.clone()),
                    MeshMaterial3d(
                        ray_assets.beam_materials[RayEyeType::Petrification.index()].clone(),
                    ),
                    Transform::from_translation(eye_pos),
                    OnGameplayScreen,
                ))
                .id();

            sweep.beam_entity = Some(beam_entity);
            sweep.glow_entity = Some(glow_entity);
        }

        if let Some(beam_entity) = sweep.beam_entity {
            if let Ok(mut beam) = beams.get_mut(beam_entity) {
                beam.origin = eye_pos;
                beam.channel_progress += delta / PETRIFY_CHANNEL_TIME;

                // Track target during channel: steer toward nearest defender
                if !beam.has_fired {
                    if let Some(target_pos) =
                        find_nearest_defender_position(boss_pos, &defenders, &team_query)
                    {
                        let desired = (target_pos - eye_pos).normalize_or_zero();
                        if desired != Vec3::ZERO {
                            let current = beam.direction;
                            let dot = current.dot(desired).clamp(-1.0, 1.0);
                            let angle = dot.acos();
                            let max_turn = PETRIFY_TURN_RATE * delta;
                            beam.direction = if angle <= max_turn {
                                desired
                            } else {
                                let t = max_turn / angle;
                                (current.lerp(desired, t)).normalize_or_zero()
                            };
                            beam.length = (target_pos - eye_pos).length().max(1.0);
                        }
                    }
                }

                if beam.channel_progress >= 1.0 && !beam.has_fired {
                    beam.has_fired = true;

                    let volume = ray_sfx_volume(eye_pos, &game_config);
                    if volume > 0.0 {
                        commands.spawn((
                            bevy::audio::AudioPlayer::new(sfx_assets.finger_of_death_cast.clone()),
                            bevy::audio::PlaybackSettings::ONCE
                                .with_volume(bevy::audio::Volume::Linear(volume)),
                            OnGameplayScreen,
                        ));
                    }

                    let cone_base_radius = PETRIFY_BEAM_WIDTH * 0.15 * 0.7;
                    let targets = find_units_in_cone(
                        eye_pos,
                        beam.direction,
                        beam.length,
                        cone_base_radius,
                        &defenders,
                        &team_query,
                    );
                    for &entity in &targets {
                        commands
                            .entity(entity)
                            .insert(Petrified::new(PETRIFY_DURATION));
                    }

                    despawn_petrify_beam(&mut commands, &mut sweep);
                    sweep.cooldown = PETRIFY_BEAM_COOLDOWN;
                }
            }
        }
    }
}

pub fn update_ray_petrification_visuals(
    mut beam_query: Query<(
        &RayPetrificationBeam,
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
    mut glow_query: Query<(&RayPetrificationGlow, &mut Transform), Without<RayPetrificationBeam>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (beam, mut transform, mat_handle) in &mut beam_query {
        let growth = beam.channel_progress.min(1.0);
        let visual_len = beam.length * growth;
        let beam_width = PETRIFY_BEAM_WIDTH * growth;

        transform.rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);
        transform.translation = beam.origin + beam.direction * visual_len / 2.0;
        transform.scale = Vec3::new(beam_width, visual_len, beam_width);

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let intensity = growth * growth;
            mat.emissive = LinearRgba::new(1.5 * intensity, 1.5 * intensity, 1.5 * intensity, 1.0);
            mat.base_color = Color::srgba(0.6, 0.6, 0.6, 0.3 + 0.4 * intensity);
            mat.alpha_mode = AlphaMode::Blend;
        }
    }

    for (glow, mut transform) in &mut glow_query {
        if let Ok((beam, beam_tf, _)) = beam_query.get(glow.beam_entity) {
            let growth = beam.channel_progress.min(1.0);
            let visual_len = beam.length * growth;
            let glow_width = PETRIFY_BEAM_WIDTH * growth * 1.3;

            transform.rotation = beam_tf.rotation;
            transform.translation = beam.origin + beam.direction * visual_len / 2.0;
            transform.scale = Vec3::new(glow_width, visual_len, glow_width);
        }
    }
}

// ===== Charm Beam =====

#[allow(clippy::too_many_arguments)]
pub fn ray_mind_control_beam(
    time: Res<Time>,
    mut commands: Commands,
    mut bosses: Query<
        (&Transform, &RayEyeState, &mut RayMindControlSweep),
        (With<Ray>, Without<Corpse>, Without<RayEye>),
    >,
    eye_query: Query<(&Transform, &RayEye)>,
    defenders: Query<
        (Entity, &Transform, &Hitbox),
        (
            With<Team>,
            Without<Corpse>,
            Without<Boss>,
            Without<RayEye>,
            Without<King>,
            Without<crate::game::units::components::KingsGuard>,
            Without<Petrified>,
        ),
    >,
    team_query: Query<&Team>,
    defender_positions: Query<
        (Entity, &Transform, Has<FearModifier>),
        (
            With<Team>,
            Without<Corpse>,
            Without<Boss>,
            Without<RayEye>,
            Without<King>,
            Without<crate::game::units::components::KingsGuard>,
            Without<Petrified>,
        ),
    >,
    ray_assets: Res<RayAssets>,
    spell_assets: Res<SpellVisualAssets>,
    sfx_assets: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    mut beams: Query<&mut RayMindControlBeam>,
) {
    let delta = time.delta_secs();

    for (boss_transform, eye_state, mut sweep) in &mut bosses {
        let boss_pos = boss_transform.translation;

        if !eye_state.active[RayEyeType::MindControl.index()] {
            despawn_mind_control_beam(&mut commands, &mut sweep);
            continue;
        }

        if sweep.beam_entity.is_none() && sweep.cooldown > 0.0 {
            sweep.cooldown -= delta;
            continue;
        }

        let has_targets = defender_positions.iter().any(|(entity, def_transform, _)| {
            if let Ok(team) = team_query.get(entity) {
                if *team != Team::Defenders {
                    return false;
                }
            }
            let horizontal = Vec2::new(
                def_transform.translation.x - boss_pos.x,
                def_transform.translation.z - boss_pos.z,
            );
            horizontal.length() <= MAX_BEAM_RANGE
        });

        if !has_targets {
            despawn_mind_control_beam(&mut commands, &mut sweep);
            continue;
        }

        let eye_pos = eye_query
            .iter()
            .find(|(_, eye)| eye.eye_type == RayEyeType::MindControl)
            .map(|(tf, _)| tf.translation)
            .unwrap_or(boss_pos);

        if sweep.beam_entity.is_none() {
            let target_pos = find_nearest_defender_position_filtered(
                boss_pos,
                &defender_positions,
                &team_query,
            );
            let target = target_pos.unwrap_or(Vec3::new(boss_pos.x - 100.0, 0.0, boss_pos.z));
            let to_target = target - eye_pos;
            let dir = to_target.normalize_or_zero();
            let length = to_target.length().max(1.0);

            let beam = RayMindControlBeam::new(eye_pos, dir, length);

            let beam_entity = commands
                .spawn((
                    beam,
                    Mesh3d(spell_assets.disintegrate_cone.clone()),
                    MeshMaterial3d(ray_assets.beam_materials[RayEyeType::MindControl.index()].clone()),
                    Transform::from_translation(eye_pos),
                    OnGameplayScreen,
                ))
                .id();

            let glow_entity = commands
                .spawn((
                    RayMindControlGlow { beam_entity },
                    Mesh3d(spell_assets.disintegrate_cone.clone()),
                    MeshMaterial3d(ray_assets.beam_materials[RayEyeType::MindControl.index()].clone()),
                    Transform::from_translation(eye_pos),
                    OnGameplayScreen,
                ))
                .id();

            sweep.beam_entity = Some(beam_entity);
            sweep.glow_entity = Some(glow_entity);
        }

        if let Some(beam_entity) = sweep.beam_entity {
            if let Ok(mut beam) = beams.get_mut(beam_entity) {
                beam.origin = eye_pos;
                beam.channel_progress += delta / MIND_CONTROL_CHANNEL_TIME;

                // Track target during channel
                if !beam.has_fired {
                    if let Some(target_pos) = find_nearest_defender_position_filtered(
                        boss_pos,
                        &defender_positions,
                        &team_query,
                    ) {
                        let desired = (target_pos - eye_pos).normalize_or_zero();
                        if desired != Vec3::ZERO {
                            let current = beam.direction;
                            let dot = current.dot(desired).clamp(-1.0, 1.0);
                            let angle = dot.acos();
                            let max_turn = MIND_CONTROL_TURN_RATE * delta;
                            beam.direction = if angle <= max_turn {
                                desired
                            } else {
                                let t = max_turn / angle;
                                (current.lerp(desired, t)).normalize_or_zero()
                            };
                            beam.length = (target_pos - eye_pos).length().max(1.0);
                        }
                    }
                }

                if beam.channel_progress >= 1.0 && !beam.has_fired {
                    beam.has_fired = true;

                    let volume = ray_sfx_volume(eye_pos, &game_config);
                    if volume > 0.0 {
                        commands.spawn((
                            bevy::audio::AudioPlayer::new(sfx_assets.mind_control_cast.clone()),
                            bevy::audio::PlaybackSettings::ONCE
                                .with_volume(bevy::audio::Volume::Linear(volume)),
                            OnGameplayScreen,
                        ));
                    }

                    let cone_base_radius = MIND_CONTROL_BEAM_WIDTH * 0.15 * 0.7;
                    let targets = find_units_in_cone_filtered(
                        eye_pos,
                        beam.direction,
                        beam.length,
                        cone_base_radius,
                        &defenders,
                        &team_query,
                    );
                    for (entity, unit_pos) in &targets {
                        commands.entity(*entity).insert((
                            MindControlled {
                                time_elapsed: 0.0,
                                wear_off_duration: MIND_CONTROL_DURATION,
                                original_spawn_pos: Some(Vec2::new(unit_pos.x, unit_pos.z)),
                                damage_multiplier: 1.0,
                            },
                            FlowFieldInfluence::Attacker,
                        ));
                    }

                    despawn_mind_control_beam(&mut commands, &mut sweep);
                    sweep.cooldown = MIND_CONTROL_BEAM_COOLDOWN;
                }
            }
        }
    }
}

fn despawn_mind_control_beam(commands: &mut Commands, sweep: &mut RayMindControlSweep) {
    if let Some(entity) = sweep.beam_entity.take() {
        commands.entity(entity).try_despawn();
    }
    if let Some(entity) = sweep.glow_entity.take() {
        commands.entity(entity).try_despawn();
    }
}

pub fn update_ray_mind_control_visuals(
    mut beam_query: Query<(&RayMindControlBeam, &mut Transform, &mut MeshMaterial3d<StandardMaterial>)>,
    mut glow_query: Query<(&RayMindControlGlow, &mut Transform), Without<RayMindControlBeam>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (beam, mut transform, mat_handle) in &mut beam_query {
        let growth = beam.channel_progress.min(1.0);
        let visual_len = beam.length * growth;
        let beam_width = MIND_CONTROL_BEAM_WIDTH * growth;

        transform.rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);
        transform.translation = beam.origin + beam.direction * visual_len / 2.0;
        transform.scale = Vec3::new(beam_width, visual_len, beam_width);

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let intensity = growth * growth;
            mat.emissive = LinearRgba::new(
                2.0 * intensity,
                0.5 * intensity,
                1.2 * intensity,
                1.0,
            );
            mat.base_color = Color::srgba(1.0, 0.3, 0.6, 0.3 + 0.4 * intensity);
            mat.alpha_mode = AlphaMode::Blend;
        }
    }

    for (glow, mut transform) in &mut glow_query {
        if let Ok((beam, beam_tf, _)) = beam_query.get(glow.beam_entity) {
            let growth = beam.channel_progress.min(1.0);
            let visual_len = beam.length * growth;
            let glow_width = MIND_CONTROL_BEAM_WIDTH * growth * 1.3;

            transform.rotation = beam_tf.rotation;
            transform.translation = beam.origin + beam.direction * visual_len / 2.0;
            transform.scale = Vec3::new(glow_width, visual_len, glow_width);
        }
    }
}

/// Nearest defender position, filtered query (excludes King and KingsGuard).
fn find_nearest_defender_position_filtered(
    boss_pos: Vec3,
    defenders: &Query<
        (Entity, &Transform, Has<FearModifier>),
        (
            With<Team>,
            Without<Corpse>,
            Without<Boss>,
            Without<RayEye>,
            Without<King>,
            Without<crate::game::units::components::KingsGuard>,
            Without<Petrified>,
        ),
    >,
    team_query: &Query<&Team>,
) -> Option<Vec3> {
    // Priority: non-feared first (0), feared second (1), then by distance.
    let mut best: Option<(Vec3, f32, u8)> = None;
    for (entity, transform, has_fear) in defenders.iter() {
        if let Ok(team) = team_query.get(entity) {
            if *team != Team::Defenders {
                continue;
            }
        }
        let to = Vec2::new(
            transform.translation.x - boss_pos.x,
            transform.translation.z - boss_pos.z,
        );
        let dist = to.length();
        if dist > MAX_BEAM_RANGE {
            continue;
        }
        let priority = if has_fear { 1 } else { 0 };
        let replace = match &best {
            None => true,
            Some((_, best_dist, best_priority)) => {
                priority < *best_priority
                    || (priority == *best_priority && dist < *best_dist)
            }
        };
        if replace {
            best = Some((transform.translation, dist, priority));
        }
    }
    best.map(|(pos, _, _)| pos)
}

/// Cone-cylinder intersection returning entity + position, filtered (excludes King, KingsGuard).
fn find_units_in_cone_filtered(
    origin: Vec3,
    direction: Vec3,
    length: f32,
    base_radius: f32,
    defenders: &Query<
        (Entity, &Transform, &Hitbox),
        (
            With<Team>,
            Without<Corpse>,
            Without<Boss>,
            Without<RayEye>,
            Without<King>,
            Without<crate::game::units::components::KingsGuard>,
            Without<Petrified>,
        ),
    >,
    team_query: &Query<&Team>,
) -> Vec<(Entity, Vec3)> {
    let mut hits = Vec::new();
    let dir_norm = direction.normalize_or_zero();

    for (entity, transform, hitbox) in defenders.iter() {
        if let Ok(team) = team_query.get(entity) {
            if *team != Team::Defenders {
                continue;
            }
        }
        let to_unit = transform.translation - origin;
        let forward_dist = to_unit.dot(dir_norm);
        if forward_dist < 0.0 || forward_dist > length {
            continue;
        }

        let closest_on_axis = origin + dir_norm * forward_dist;
        let perp_dist = (transform.translation - closest_on_axis).length();

        let cone_t = forward_dist / length;
        let cone_radius = base_radius * cone_t;

        if perp_dist <= cone_radius + hitbox.radius {
            hits.push((entity, transform.translation));
        }
    }
    hits
}

// ===== Petrified Unit Effects =====

pub fn update_petrified_damage(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<crate::game::units::king::components::SpellShield>,
        ),
        (With<Petrified>, With<King>),
    >,
) {
    let delta = time.delta_secs();
    let damage = PETRIFY_KING_DAMAGE_PER_SECOND * delta;

    for (entity, mut health, temp_hp, has_shield) in &mut query {
        apply_spell_damage(
            &mut commands,
            entity,
            &mut health,
            temp_hp.map(|t| t.into_inner()),
            damage,
            DamageType::Force,
            has_shield,
        );
    }
}

// ===== Fear Beam =====

#[allow(clippy::too_many_arguments)]
pub fn ray_fear_beam(
    time: Res<Time>,
    mut commands: Commands,
    mut bosses: Query<
        (&Transform, &RayEyeState, &mut RayFearSweep),
        (With<Ray>, Without<Corpse>, Without<RayEye>),
    >,
    eye_query: Query<(&Transform, &RayEye)>,
    defenders: Query<
        (Entity, &Transform, &Hitbox, Has<BerserkerRageModifier>, Has<MindControlled>),
        (
            With<Team>,
            Without<Corpse>,
            Without<Boss>,
            Without<RayEye>,
            Without<King>,
        ),
    >,
    team_query: Query<&Team>,
    ray_assets: Res<RayAssets>,
    spell_assets: Res<SpellVisualAssets>,
    mut beams: Query<&mut RayFearBeam>,
) {
    let delta = time.delta_secs();

    for (boss_transform, eye_state, mut sweep) in &mut bosses {
        let boss_pos = boss_transform.translation;

        if !eye_state.active[RayEyeType::Fear.index()] {
            despawn_fear_beam(&mut commands, &mut sweep);
            continue;
        }

        let eye_pos = eye_query
            .iter()
            .find(|(_, eye)| eye.eye_type == RayEyeType::Fear)
            .map(|(tf, _)| tf.translation)
            .unwrap_or(boss_pos);

        // Spawn beam if needed
        if sweep.beam_entity.is_none() {
            let length = eye_pos.y;
            let beam = RayFearBeam {
                origin: eye_pos,
                length,
                time_alive: 0.0,
            };

            let beam_entity = commands
                .spawn((
                    beam,
                    Mesh3d(spell_assets.disintegrate_cone.clone()),
                    MeshMaterial3d(ray_assets.beam_materials[RayEyeType::Fear.index()].clone()),
                    Transform::from_translation(eye_pos),
                    OnGameplayScreen,
                ))
                .id();

            let glow_entity = commands
                .spawn((
                    RayFearGlow { beam_entity },
                    Mesh3d(spell_assets.disintegrate_cone.clone()),
                    MeshMaterial3d(ray_assets.beam_materials[RayEyeType::Fear.index()].clone()),
                    Transform::from_translation(eye_pos),
                    OnGameplayScreen,
                ))
                .id();

            sweep.beam_entity = Some(beam_entity);
            sweep.glow_entity = Some(glow_entity);
        }

        // Update beam position + apply fear
        if let Some(beam_entity) = sweep.beam_entity {
            if let Ok(mut beam) = beams.get_mut(beam_entity) {
                beam.origin = eye_pos;
                beam.length = eye_pos.y.max(1.0);
                beam.time_alive += delta;

                // Apply fear to units under the beam
                sweep.fear_cooldown -= delta;
                if sweep.fear_cooldown <= 0.0 {
                    sweep.fear_cooldown = FEAR_BEAM_COOLDOWN;

                    let ground_pos = Vec2::new(eye_pos.x, eye_pos.z);
                    for (entity, def_tf, hitbox, has_rage, is_mind_controlled) in &defenders {
                        if has_rage || is_mind_controlled {
                            continue;
                        }
                        if let Ok(team) = team_query.get(entity) {
                            if *team != Team::Defenders {
                                continue;
                            }
                        }
                        let unit_xz = Vec2::new(def_tf.translation.x, def_tf.translation.z);
                        let dist = (unit_xz - ground_pos).length();
                        if dist <= FEAR_BEAM_GROUND_RADIUS + hitbox.radius {
                            commands
                                .entity(entity)
                                .insert(FearModifier::new(FEAR_DURATION, boss_pos));
                        }
                    }
                }
            }
        }
    }
}

fn despawn_fear_beam(commands: &mut Commands, sweep: &mut RayFearSweep) {
    if let Some(entity) = sweep.beam_entity.take() {
        commands.entity(entity).try_despawn();
    }
    if let Some(entity) = sweep.glow_entity.take() {
        commands.entity(entity).try_despawn();
    }
}

pub fn update_ray_fear_visuals(
    time: Res<Time>,
    mut beam_query: Query<(
        &RayFearBeam,
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
    mut glow_query: Query<(&RayFearGlow, &mut Transform), Without<RayFearBeam>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let t = time.elapsed_secs();

    for (beam, mut transform, mat_handle) in &mut beam_query {
        let visual_len = beam.length;
        let pulse = 1.0 + 0.1 * (t * 6.0 * std::f32::consts::TAU).sin();
        let cone_scale = (FEAR_BEAM_GROUND_RADIUS / 0.15) * pulse;

        transform.rotation = Quat::from_rotation_arc(Vec3::Y, Vec3::NEG_Y);
        transform.translation = beam.origin + Vec3::NEG_Y * visual_len / 2.0;
        transform.scale = Vec3::new(cone_scale, visual_len, cone_scale);

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let cycle = (t * 3.0).sin() * 0.5 + 0.5;
            mat.emissive = LinearRgba::new(1.5 + cycle * 1.0, 0.0, 2.5 + cycle * 1.5, 1.0);
            mat.base_color = Color::srgba(0.6, 0.0, 0.8, 0.5);
            mat.alpha_mode = AlphaMode::Blend;
        }
    }

    for (glow, mut transform) in &mut glow_query {
        if let Ok((beam, beam_tf, _)) = beam_query.get(glow.beam_entity) {
            let visual_len = beam.length;
            let glow_scale = (FEAR_BEAM_GROUND_RADIUS / 0.15) * 1.3;

            transform.rotation = beam_tf.rotation;
            transform.translation = beam.origin + Vec3::NEG_Y * visual_len / 2.0;
            transform.scale = Vec3::new(glow_scale, visual_len, glow_scale);
        }
    }
}

// ===== Teleport Eye =====

#[allow(clippy::too_many_arguments)]
pub fn ray_teleport_eye(
    time: Res<Time>,
    mut commands: Commands,
    mut bosses: Query<
        (&Transform, &RayEyeState, &mut RayTeleportSweep),
        (With<Ray>, Without<Corpse>, Without<RayEye>),
    >,
    eye_query: Query<(&Transform, &RayEye)>,
    defenders: Query<
        (Entity, &Transform, Has<Petrified>, Has<FearModifier>, Has<MindControlled>),
        (
            With<Team>,
            Without<Corpse>,
            Without<Boss>,
            Without<RayEye>,
            Without<King>,
        ),
    >,
    team_query: Query<&Team>,
    spell_assets: Res<SpellVisualAssets>,
    sfx_assets: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_rng: ResMut<GameRng>,
) {
    let delta = time.delta_secs();

    for (boss_transform, eye_state, mut sweep) in &mut bosses {
        if !eye_state.active[RayEyeType::Teleportation.index()] {
            continue;
        }

        sweep.cooldown -= delta;
        if sweep.cooldown > 0.0 {
            continue;
        }

        let boss_pos = boss_transform.translation;

        // Only activate when a defender is in melee range of Ray's body
        let any_in_melee = defenders.iter().any(|(entity, def_tf, _, _, _)| {
            if let Ok(team) = team_query.get(entity) {
                if *team != Team::Defenders {
                    return false;
                }
            }
            let dist = Vec2::new(
                def_tf.translation.x - boss_pos.x,
                def_tf.translation.z - boss_pos.z,
            )
            .length();
            dist <= MELEE_SLOWDOWN_DISTANCE
        });

        if !any_in_melee {
            continue;
        }

        let eye_pos = eye_query
            .iter()
            .find(|(_, eye)| eye.eye_type == RayEyeType::Teleportation)
            .map(|(tf, _)| tf.translation)
            .unwrap_or(boss_pos);

        // Find closest 50 defenders (excluding King, petrified, and charmed;
        // prioritizing non-feared / non-charmed / non-petrified)
        let mut defender_dists: Vec<(Entity, f32, u8)> = defenders
            .iter()
            .filter_map(|(entity, def_tf, is_petrified, has_fear, is_charmed)| {
                if is_petrified || is_charmed {
                    return None;
                }
                if let Ok(team) = team_query.get(entity) {
                    if *team != Team::Defenders {
                        return None;
                    }
                }
                let dist = Vec2::new(
                    def_tf.translation.x - boss_pos.x,
                    def_tf.translation.z - boss_pos.z,
                )
                .length();
                // Priority: 0 = non-feared (best), 1 = feared
                let priority = if has_fear { 1 } else { 0 };
                Some((entity, dist, priority))
            })
            .collect();

        if defender_dists.is_empty() {
            continue;
        }

        // Sort: priority first (non-feared), then by distance
        defender_dists.sort_by(|a, b| {
            a.2.cmp(&b.2)
                .then_with(|| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        });
        defender_dists.truncate(TELEPORT_EYE_UNIT_COUNT);

        let rng = &mut game_rng.0;
        for &(entity, _, _) in &defender_dists {
            let scatter_x = rng.gen_range(VISIBLE_MIN_X * 0.6..VISIBLE_MAX_X * 0.6);
            let scatter_z = rng.gen_range(VISIBLE_MIN_Z * 0.6..VISIBLE_MAX_Z * 0.6);
            if let Ok((_, def_tf, _, _, _)) = defenders.get(entity) {
                commands.entity(entity).insert(Transform::from_xyz(
                    scatter_x,
                    def_tf.translation.y,
                    scatter_z,
                ));
            }
        }

        // Spawn expanding bubble VFX at the eye
        let bubble_mat = materials.add(StandardMaterial {
            base_color: Color::srgba(0.2, 0.5, 1.0, 0.2),
            emissive: LinearRgba::new(0.5, 1.0, 2.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        });

        commands.spawn((
            Mesh3d(spell_assets.explosion_sphere.clone()),
            MeshMaterial3d(bubble_mat),
            Transform::from_translation(eye_pos),
            RayTeleportBubble { time_alive: 0.0 },
            OnGameplayScreen,
        ));

        let volume = ray_sfx_volume(eye_pos, &game_config);
        if volume > 0.0 {
            commands.spawn((
                bevy::audio::AudioPlayer::new(sfx_assets.teleport_cast.clone()),
                bevy::audio::PlaybackSettings::ONCE
                    .with_volume(bevy::audio::Volume::Linear(volume)),
                OnGameplayScreen,
            ));
        }

        sweep.cooldown = TELEPORT_EYE_COOLDOWN;
    }
}

pub fn update_ray_teleport_bubbles(
    time: Res<Time>,
    mut commands: Commands,
    mut bubbles: Query<(
        Entity,
        &mut RayTeleportBubble,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();

    for (entity, mut bubble, mut transform, mat_handle) in &mut bubbles {
        bubble.time_alive += delta;

        if bubble.time_alive >= TELEPORT_BUBBLE_DURATION {
            commands.entity(entity).try_despawn();
            continue;
        }

        let radius = TELEPORT_BUBBLE_EXPAND_SPEED * bubble.time_alive;
        transform.scale = Vec3::splat(radius);

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let fade = 1.0 - (bubble.time_alive / TELEPORT_BUBBLE_DURATION);
            mat.base_color = Color::srgba(0.2, 0.5, 1.0, 0.2 * fade);
        }
    }
}

/// 3D cone-cylinder intersection. The cone originates at `origin` with radius 0,
/// widens linearly to `base_radius` at `length` along `direction`.
/// Projects each unit onto the 3D beam axis and checks perpendicular distance
/// against the cone radius at that depth.
fn find_units_in_cone(
    origin: Vec3,
    direction: Vec3,
    length: f32,
    base_radius: f32,
    defenders: &Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &mut Health,
            Option<&SpellShield>,
            Option<&mut TemporaryHitPoints>,
        ),
        (With<Team>, Without<Corpse>, Without<Boss>, Without<RayEye>),
    >,
    team_query: &Query<&Team>,
) -> Vec<Entity> {
    let mut hits = Vec::new();
    let dir_norm = direction.normalize_or_zero();

    for (entity, transform, hitbox, _, _, _) in defenders.iter() {
        if let Ok(team) = team_query.get(entity) {
            if *team != Team::Defenders {
                continue;
            }
        }

        // Project unit center onto the 3D beam axis
        let to_unit = transform.translation - origin;
        let forward_dist = to_unit.dot(dir_norm);
        if forward_dist < 0.0 || forward_dist > length {
            continue;
        }

        // Perpendicular distance from the 3D beam axis
        let closest_on_axis = origin + dir_norm * forward_dist;
        let perp_dist = (transform.translation - closest_on_axis).length();

        // Cone radius widens linearly from 0 at origin to base_radius at length
        let cone_radius_at_dist = (forward_dist / length) * base_radius;

        if perp_dist <= cone_radius_at_dist + hitbox.radius {
            hits.push(entity);
        }
    }
    hits
}

/// Find direction from `from_pos` (XZ) to nearest defender. Used for reticle steering.
fn find_nearest_defender_direction_from(
    from_pos: Vec2,
    defenders: &Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &mut Health,
            Option<&SpellShield>,
            Option<&mut TemporaryHitPoints>,
        ),
        (With<Team>, Without<Corpse>, Without<Boss>, Without<RayEye>),
    >,
    team_query: &Query<&Team>,
) -> Option<Vec2> {
    let mut best: Option<(Vec2, f32)> = None;
    for (entity, transform, _, _, _, _) in defenders.iter() {
        if let Ok(team) = team_query.get(entity) {
            if *team != Team::Defenders {
                continue;
            }
        }
        let to = Vec2::new(
            transform.translation.x - from_pos.x,
            transform.translation.z - from_pos.y,
        );
        let dist = to.length();
        if dist < 1.0 {
            continue;
        }
        match &best {
            Some((_, best_dist)) if dist >= *best_dist => {}
            _ => best = Some((to, dist)),
        }
    }
    best.map(|(to, _)| to.normalize_or_zero())
}

// ===== Ray Beam Visuals =====

pub fn update_ray_disintegrate_visuals(
    time: Res<Time>,
    mut beam_query: Query<(
        &RayDisintegrateBeam,
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
    mut glow_query: Query<(&RayDisintegrateGlow, &mut Transform), Without<RayDisintegrateBeam>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let t = time.elapsed_secs();

    for (beam, mut transform, mat_handle) in &mut beam_query {
        let growth = (beam.time_alive / DISINTEGRATION_BEAM_GROWTH_TIME).min(1.0);
        let visual_len = beam.length * growth;
        let pulse = 1.0
            + DISINTEGRATION_BEAM_PULSE_AMOUNT
                * (t * DISINTEGRATION_BEAM_PULSE_SPEED * std::f32::consts::TAU).sin();
        let beam_width = BEAM_WIDTH * pulse;

        transform.rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);
        transform.translation = beam.origin + beam.direction * visual_len / 2.0;
        transform.scale = Vec3::new(beam_width, visual_len, beam_width);

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            let cycle = (t * 4.0).sin() * 0.5 + 0.5;
            mat.emissive =
                LinearRgba::new(3.0 + cycle * 2.0, 1.5 + cycle * 3.0, 0.2 + cycle * 3.8, 1.0);
            mat.base_color = Color::srgba(1.0, 0.6 + cycle * 0.35, 0.1 + cycle * 0.6, 0.5);
            mat.alpha_mode = AlphaMode::Blend;
        }
    }

    for (glow, mut transform) in &mut glow_query {
        if let Ok((beam, beam_tf, _)) = beam_query.get(glow.beam_entity) {
            let growth = (beam.time_alive / DISINTEGRATION_BEAM_GROWTH_TIME).min(1.0);
            let visual_len = beam.length * growth;
            let glow_width = BEAM_WIDTH * 1.5;

            transform.rotation = beam_tf.rotation;
            transform.translation = beam.origin + beam.direction * visual_len / 2.0;
            transform.scale = Vec3::new(glow_width, visual_len, glow_width);
        }
    }
}

fn find_nearest_defender_position(
    boss_pos: Vec3,
    defenders: &Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &mut Health,
            Option<&SpellShield>,
            Option<&mut TemporaryHitPoints>,
        ),
        (With<Team>, Without<Corpse>, Without<Boss>, Without<RayEye>),
    >,
    team_query: &Query<&Team>,
) -> Option<Vec3> {
    let mut best: Option<(Vec3, f32)> = None;
    for (entity, transform, _, _, _, _) in defenders.iter() {
        if let Ok(team) = team_query.get(entity) {
            if *team != Team::Defenders {
                continue;
            }
        }
        let to = Vec2::new(
            transform.translation.x - boss_pos.x,
            transform.translation.z - boss_pos.z,
        );
        let dist = to.length();
        if dist > MAX_BEAM_RANGE {
            continue;
        }
        match &best {
            Some((_, best_dist)) if dist >= *best_dist => {}
            _ => best = Some((transform.translation, dist)),
        }
    }
    best.map(|(pos, _)| pos)
}

