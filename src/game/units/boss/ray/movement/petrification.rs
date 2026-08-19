use bevy::prelude::*;

use super::super::beams::{find_nearest_defender_position, find_units_in_cone};
use super::super::components::*;
use super::super::constants::*;
use super::super::resources::RayAssets;
use super::spawn::ray_sfx_volume;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{Corpse, Health, Hitbox, Petrified, Team, TemporaryHitPoints};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::audio::SpellSfxAssets;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

pub(crate) fn despawn_petrify_beam(commands: &mut Commands, sweep: &mut RayPetrificationSweep) {
    if let Some(entity) = sweep.beam_entity.take() {
        commands.entity(entity).try_despawn();
    }
    if let Some(entity) = sweep.glow_entity.take() {
        commands.entity(entity).try_despawn();
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
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
        (
            With<Team>,
            Without<Corpse>,
            Without<Boss>,
            Without<RayEye>,
            Without<Wizard>,
        ),
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

        if let Some(beam_entity) = sweep.beam_entity
            && let Ok(mut beam) = beams.get_mut(beam_entity)
        {
            beam.origin = eye_pos;
            beam.channel_progress += delta / PETRIFY_CHANNEL_TIME;

            // Track target during channel: steer toward nearest defender
            if !beam.has_fired
                && let Some(target_pos) =
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

        if let Some(mut mat) = materials.get_mut(&mat_handle.0) {
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
