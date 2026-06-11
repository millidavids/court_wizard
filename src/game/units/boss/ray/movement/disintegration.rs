use bevy::prelude::*;
use rand::Rng;

use super::super::beams::find_units_in_cone;
use super::super::beams::{find_nearest_defender_direction_from, find_nearest_defender_position};
use super::super::components::*;
use super::super::constants::*;
use super::super::resources::RayAssets;
use super::spawn::ray_sfx_volume;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::seeded_rng::resources::GameRng;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::apply_spell_damage;
use crate::game::units::components::{Corpse, Health, Hitbox, Team, TemporaryHitPoints};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::audio::SpellSfxAssets;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

pub(crate) fn despawn_ray_beam(commands: &mut Commands, sweep: &mut RayDisintegrationSweep) {
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

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
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
            if let Ok(team) = team_query.get(entity)
                && *team != Team::Defenders
            {
                return false;
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
            let angle = rng.random::<f32>() * std::f32::consts::TAU;
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
