use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{Corpse, FearModifier, Hitbox, Petrified, Team};
use crate::game::units::king::components::King;

/// Nearest defender position, filtered query (excludes King and KingsGuard).
#[allow(clippy::type_complexity)]
pub(crate) fn find_nearest_defender_position_filtered(
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
pub(crate) fn find_units_in_cone_filtered(
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
