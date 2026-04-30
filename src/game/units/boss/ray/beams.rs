//! Ray beam attacks: mind control, fear, teleport, petrification visuals.

use super::movement::ray_sfx_volume;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use super::resources::RayAssets;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::*;
use crate::game::pathfinding::FlowFieldInfluence;
use crate::game::seeded_rng::resources::GameRng;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::apply_spell_damage;
use crate::game::units::components::{
    BerserkerRageModifier, Corpse, FearModifier, Health, Hitbox, MindControlled, Petrified, Team,
    TemporaryHitPoints,
};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::{King, SpellShield};
use crate::game::units::wizard::spells::audio::SpellSfxAssets;

/// Attenuated volume for Ray's sound effects — slight falloff from wizard/camera position.
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

pub(super) fn despawn_mind_control_beam(commands: &mut Commands, sweep: &mut RayMindControlSweep) {
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

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
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

/// Nearest defender position, filtered query (excludes King and KingsGuard).
#[allow(clippy::type_complexity)]
pub(super) fn find_nearest_defender_position_filtered(
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
        if let Ok(team) = team_query.get(entity)
            && *team != Team::Defenders
        {
            continue;
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
                priority < *best_priority || (priority == *best_priority && dist < *best_dist)
            }
        };
        if replace {
            best = Some((transform.translation, dist, priority));
        }
    }
    best.map(|(pos, _, _)| pos)
}

/// Cone-cylinder intersection returning entity + position, filtered (excludes King, KingsGuard).
#[allow(clippy::type_complexity)]
pub(super) fn find_units_in_cone_filtered(
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
        if let Ok(team) = team_query.get(entity)
            && *team != Team::Defenders
        {
            continue;
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

pub(super) fn despawn_fear_beam(commands: &mut Commands, sweep: &mut RayFearSweep) {
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

/// 3D cone-cylinder intersection. The cone originates at `origin` with radius 0,
/// widens linearly to `base_radius` at `length` along `direction`.
/// Projects each unit onto the 3D beam axis and checks perpendicular distance
/// against the cone radius at that depth.
#[allow(clippy::type_complexity)]
pub(super) fn find_units_in_cone(
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
        if let Ok(team) = team_query.get(entity)
            && *team != Team::Defenders
        {
            continue;
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
#[allow(clippy::type_complexity)]
pub(super) fn find_nearest_defender_direction_from(
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
        if let Ok(team) = team_query.get(entity)
            && *team != Team::Defenders
        {
            continue;
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

#[allow(clippy::type_complexity)]
pub(super) fn find_nearest_defender_position(
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
        if let Ok(team) = team_query.get(entity)
            && *team != Team::Defenders
        {
            continue;
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
