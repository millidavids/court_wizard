use std::cmp::Ordering;

use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::units::components::Corpse;
use crate::game::units::components::Team;
use crate::game::units::wizard::spells::arcane_crystal::components::ArcaneCrystal;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

use super::cast::MissileParams;

/// Spawns a magic missile with talent modifications applied.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_magic_missile_with_talents(
    rng: &mut impl Rng,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    camera_query: &Query<&GlobalTransform, With<Camera>>,
    targets: &Query<(Entity, &Transform, &Team), (Without<MagicMissile>, Without<Corpse>)>,
    crystals: &Query<(Entity, &Transform, &ArcaneCrystal)>,
    spell_range: f32,
    empowerment: f32,
    cursor_world_pos: Option<Vec3>,
    spawn_origin: Vec3,
    target_teams: TargetTeams,
    params: &MissileParams,
) {
    let spawn_pos = spawn_origin + Vec3::new(0.0, constants::SPAWN_HEIGHT_OFFSET, 0.0);

    let crystal_target: Option<Entity> = cursor_world_pos.and_then(|cursor_pos| {
        let mut closest: Option<(Entity, f32)> = None;
        for (entity, transform, crystal) in crystals.iter() {
            let dist = Vec3::new(
                cursor_pos.x - transform.translation.x,
                0.0,
                cursor_pos.z - transform.translation.z,
            )
            .length();
            if dist <= crystal.collision_radius * 5.0 {
                match closest {
                    None => closest = Some((entity, dist)),
                    Some((_, prev_dist)) if dist < prev_dist => closest = Some((entity, dist)),
                    _ => {}
                }
            }
        }
        closest.map(|(e, _)| e)
    });

    // Guided missiles don't need a target — they steer toward the cursor
    let target = if params.guided {
        None
    } else if crystal_target.is_some() {
        crystal_target
    } else {
        let enemies_in_range: Vec<Entity> = targets
            .iter()
            .filter(|(_, _, team)| target_teams.matches(team))
            .filter(|(_, transform, _)| {
                let distance = spawn_pos.distance(transform.translation);
                distance <= spell_range
            })
            .map(|(entity, _, _)| entity)
            .collect();

        if !enemies_in_range.is_empty() {
            if let Some(cursor_pos) = cursor_world_pos {
                let mut total_weight = 0.0;
                let weighted_targets: Vec<(Entity, f32)> = enemies_in_range
                    .iter()
                    .filter_map(|&entity| {
                        targets.get(entity).ok().map(|(_, transform, _)| {
                            let distance = cursor_pos.distance(transform.translation);
                            let weight = 1.0
                                / (distance.powi(constants::CURSOR_TARGETING_WEIGHT_POWER) + 1.0);
                            total_weight += weight;
                            (entity, weight)
                        })
                    })
                    .collect();

                if total_weight > 0.0 {
                    let mut random_value = rng.random_range(0.0..total_weight);
                    let mut selected_target = None;
                    for (entity, weight) in weighted_targets {
                        random_value -= weight;
                        if random_value <= 0.0 {
                            selected_target = Some(entity);
                            break;
                        }
                    }
                    selected_target.or_else(|| enemies_in_range.first().copied())
                } else {
                    let index = rng.random_range(0..enemies_in_range.len());
                    Some(enemies_in_range[index])
                }
            } else {
                let index = rng.random_range(0..enemies_in_range.len());
                Some(enemies_in_range[index])
            }
        } else {
            targets
                .iter()
                .filter(|(_, _, team)| target_teams.matches(team))
                .min_by(|a, b| {
                    let dist_a = spawn_pos.distance(a.1.translation);
                    let dist_b = spawn_pos.distance(b.1.translation);
                    dist_a.partial_cmp(&dist_b).unwrap_or(Ordering::Equal)
                })
                .map(|(entity, _, _)| entity)
        }
    };

    // Random initial velocity
    let horizontal_x =
        rng.random_range(constants::HORIZONTAL_VEL_MIN..constants::HORIZONTAL_VEL_MAX);
    let horizontal_z =
        rng.random_range(constants::HORIZONTAL_VEL_MIN..constants::HORIZONTAL_VEL_MAX);
    let vertical = rng.random_range(constants::VERTICAL_VEL_MIN..constants::VERTICAL_VEL_MAX);
    let mut initial_velocity = Vec3::new(horizontal_x, vertical, horizontal_z);

    // Storm wobble: increase initial velocity randomness
    if params.storm_wobble {
        initial_velocity *= 1.5;
    }

    if let Ok(camera_transform) = camera_query.single() {
        let camera_pos = camera_transform.translation();
        let to_camera = (camera_pos - spawn_pos).normalize_or_zero();
        let camera_arc_speed =
            rng.random_range(constants::CAMERA_ARC_SPEED_MIN..constants::CAMERA_ARC_SPEED_MAX);
        let camera_arc = to_camera * camera_arc_speed;
        initial_velocity += camera_arc;
    }

    let wobble_offset = rng.random_range(0.0..std::f32::consts::TAU);

    // Construct missile with modified damage and radius
    let scale = empowerment;
    let radius_mult = if params.heavy { 2.0 } else { 1.0 };

    let mut missile = MagicMissile::new(
        initial_velocity,
        wobble_offset,
        target,
        empowerment,
        target_teams,
        spell_range,
        spawn_pos,
    );
    missile.damage = constants::DAMAGE * scale * params.damage_mult;
    missile.radius = constants::COLLISION_RADIUS * scale * radius_mult;
    missile.piercing = params.piercing;
    missile.detonation = params.detonation;
    missile.seeker_swarm = params.seeker_swarm;
    missile.guided = params.guided;

    let entity = commands
        .spawn((
            Mesh3d(assets.magic_missile_mesh.clone()),
            MeshMaterial3d(assets.magic_missile.clone()),
            Transform::from_translation(spawn_pos),
            missile,
            OnGameplayScreen,
        ))
        .id();

    vfx::systems::spawn_missile_glow(
        commands,
        assets,
        entity,
        spawn_pos,
        constants::COLLISION_RADIUS * radius_mult,
    );
}
