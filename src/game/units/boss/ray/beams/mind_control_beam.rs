use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::super::movement::ray_sfx_volume;
use super::super::resources::RayAssets;
use super::beam_helpers::{find_nearest_defender_position_filtered, find_units_in_cone_filtered};
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::pathfinding::FlowFieldInfluence;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{
    Corpse, FearModifier, Hitbox, MindControlled, Petrified, Team,
};
use crate::game::units::king::components::King;
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::audio::SpellSfxAssets;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Manages Ray's mind-control beam sweep: channels, fires, and applies MindControlled to hit units.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
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
            Without<Wizard>,
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
            Without<Wizard>,
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
            despawn_mind_control_beam(&mut commands, &mut sweep);
            continue;
        }

        let eye_pos = eye_query
            .iter()
            .find(|(_, eye)| eye.eye_type == RayEyeType::MindControl)
            .map(|(tf, _)| tf.translation)
            .unwrap_or(boss_pos);

        if sweep.beam_entity.is_none() {
            let target_pos =
                find_nearest_defender_position_filtered(boss_pos, &defender_positions, &team_query);
            let target = target_pos.unwrap_or(Vec3::new(boss_pos.x - 100.0, 0.0, boss_pos.z));
            let to_target = target - eye_pos;
            let dir = to_target.normalize_or_zero();
            let length = to_target.length().max(1.0);

            let beam = RayMindControlBeam::new(eye_pos, dir, length);

            let beam_entity = commands
                .spawn((
                    beam,
                    Mesh3d(spell_assets.disintegrate_cone.clone()),
                    MeshMaterial3d(
                        ray_assets.beam_materials[RayEyeType::MindControl.index()].clone(),
                    ),
                    Transform::from_translation(eye_pos),
                    OnGameplayScreen,
                ))
                .id();

            let glow_entity = commands
                .spawn((
                    RayMindControlGlow { beam_entity },
                    Mesh3d(spell_assets.disintegrate_cone.clone()),
                    MeshMaterial3d(
                        ray_assets.beam_materials[RayEyeType::MindControl.index()].clone(),
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
            beam.channel_progress += delta / MIND_CONTROL_CHANNEL_TIME;

            // Track target during channel
            if !beam.has_fired
                && let Some(target_pos) = find_nearest_defender_position_filtered(
                    boss_pos,
                    &defender_positions,
                    &team_query,
                )
            {
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

pub(crate) fn despawn_mind_control_beam(commands: &mut Commands, sweep: &mut RayMindControlSweep) {
    if let Some(entity) = sweep.beam_entity.take() {
        commands.entity(entity).try_despawn();
    }
    if let Some(entity) = sweep.glow_entity.take() {
        commands.entity(entity).try_despawn();
    }
}

pub fn update_ray_mind_control_visuals(
    mut beam_query: Query<(
        &RayMindControlBeam,
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
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

        if let Some(mut mat) = materials.get_mut(&mat_handle.0) {
            let intensity = growth * growth;
            mat.emissive = LinearRgba::new(2.0 * intensity, 0.5 * intensity, 1.2 * intensity, 1.0);
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
