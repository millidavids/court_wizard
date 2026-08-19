use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{Corpse, Health, Hitbox, Team, TemporaryHitPoints};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Wizard;

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

        if let Some(mut mat) = materials.get_mut(&mat_handle.0) {
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

/// 3D cone-cylinder intersection. The cone originates at `origin` with radius 0,
/// widens linearly to `base_radius` at `length` along `direction`.
/// Projects each unit onto the 3D beam axis and checks perpendicular distance
/// against the cone radius at that depth.
#[allow(clippy::type_complexity)]
pub(crate) fn find_units_in_cone(
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
        (
            With<Team>,
            Without<Corpse>,
            Without<Boss>,
            Without<RayEye>,
            Without<Wizard>,
        ),
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
pub(crate) fn find_nearest_defender_direction_from(
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
        (
            With<Team>,
            Without<Corpse>,
            Without<Boss>,
            Without<RayEye>,
            Without<Wizard>,
        ),
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

#[allow(clippy::type_complexity)]
pub(crate) fn find_nearest_defender_position(
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
        (
            With<Team>,
            Without<Corpse>,
            Without<Boss>,
            Without<RayEye>,
            Without<Wizard>,
        ),
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
