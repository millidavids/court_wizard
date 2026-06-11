use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use super::super::movement::ray_sfx_volume;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::*;
use crate::game::seeded_rng::resources::GameRng;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{Corpse, FearModifier, MindControlled, Petrified, Team};
use crate::game::units::king::components::King;
use crate::game::units::wizard::spells::audio::SpellSfxAssets;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

// ===== Teleport Eye =====

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn ray_teleport_eye(
    time: Res<Time>,
    mut commands: Commands,
    mut bosses: Query<
        (&Transform, &RayEyeState, &mut RayTeleportSweep),
        (With<Ray>, Without<Corpse>, Without<RayEye>),
    >,
    eye_query: Query<(&Transform, &RayEye)>,
    defenders: Query<
        (
            Entity,
            &Transform,
            Has<Petrified>,
            Has<FearModifier>,
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
            if let Ok(team) = team_query.get(entity)
                && *team != Team::Defenders
            {
                return false;
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
                if let Ok(team) = team_query.get(entity)
                    && *team != Team::Defenders
                {
                    return None;
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
            let scatter_x = rng.random_range(VISIBLE_MIN_X * 0.6..VISIBLE_MAX_X * 0.6);
            let scatter_z = rng.random_range(VISIBLE_MIN_Z * 0.6..VISIBLE_MAX_Z * 0.6);
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
