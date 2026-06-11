use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::RayAssets;
use crate::game::components::OnGameplayScreen;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{
    BerserkerRageModifier, Corpse, FearModifier, Hitbox, MindControlled, Team,
};
use crate::game::units::king::components::King;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn ray_fear_beam(
    time: Res<Time>,
    mut commands: Commands,
    mut bosses: Query<
        (&Transform, &RayEyeState, &mut RayFearSweep),
        (With<Ray>, Without<Corpse>, Without<RayEye>),
    >,
    eye_query: Query<(&Transform, &RayEye)>,
    defenders: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            Has<BerserkerRageModifier>,
            Has<MindControlled>,
        ),
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
        if let Some(beam_entity) = sweep.beam_entity
            && let Ok(mut beam) = beams.get_mut(beam_entity)
        {
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
                    if let Ok(team) = team_query.get(entity)
                        && *team != Team::Defenders
                    {
                        continue;
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

pub(crate) fn despawn_fear_beam(commands: &mut Commands, sweep: &mut RayFearSweep) {
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
