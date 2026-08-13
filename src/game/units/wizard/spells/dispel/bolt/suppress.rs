use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::components::MindControlled;
use crate::game::units::king::components::SpellShield;
use crate::game::units::shielder::components::ShielderDamageReduction;
use crate::game::units::wizard::spells::grease::components::{GreaseIgnited, GreaseZone};
use crate::game::units::wizard::spells::meteor_fall::components::MeteorGroundFire;
use crate::game::units::wizard::spells::spike_growth::components::SpikeGrowthZone;
use crate::game::units::wizard::spells::utils::xz_distance;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

/// Returns true if the spell effect kind is dispellable.
///
/// Excludes instantaneous explosions (already detonating), the ScorchedEarth /
/// Napalm trail variants (cosmetic ground fire; not the intended dispel
/// target), and DispelImpact/SquallStorm (the former is self-driven by the
/// spell; the latter is a parent marker whose effective state lives in its
/// children).
pub(crate) fn is_dispellable(kind: SpellEffectKind) -> bool {
    !matches!(
        kind,
        SpellEffectKind::FireballExplosion
            | SpellEffectKind::MeteorExplosion
            | SpellEffectKind::IceExplosion
            | SpellEffectKind::HealingPlumeZone
            | SpellEffectKind::DispelImpact
            | SpellEffectKind::SquallStorm
            | SpellEffectKind::ScorchedEarthFire
            | SpellEffectKind::NapalmTrail
            // The crystal is removed by dispel, but through its own shatter
            // path so it detonates instead of quietly vanishing. Deleting it
            // here would beat the shatter to it.
            | SpellEffectKind::ArcaneCrystal
    )
}

/// Returns true if this spell effect kind is an offensive (damage-dealing) effect
/// for Spell Reflection purposes.
pub(crate) fn is_offensive_effect(kind: SpellEffectKind) -> bool {
    matches!(
        kind,
        SpellEffectKind::SpikeGrowthZone
            | SpellEffectKind::WallOfFire
            | SpellEffectKind::MeteorGroundFire
            | SpellEffectKind::PlagueWindCloud
            | SpellEffectKind::GreaseFire
            | SpellEffectKind::BlackHole
    )
}

/// Collects dispellable spell effects from a query into a Vec for use with `suppress_spell_effects_in_radius`.
pub(crate) fn collect_dispellable_effects(
    spell_effects: impl Iterator<Item = (Entity, Vec3, SpellEffectKind)>,
) -> Vec<(Entity, Vec3, SpellEffectKind)> {
    spell_effects
        .filter(|&(_, _, kind)| is_dispellable(kind))
        .collect()
}

/// Suppresses (despawns) all dispellable spell effects within `radius` of `center`.
/// Returns the list of (entity, position, kind) for each dispelled effect so callers
/// can apply additional talent logic (e.g. Mana Drain, Explosive Nullification).
#[allow(clippy::too_many_arguments)]
pub(crate) fn suppress_spell_effects_in_radius(
    commands: &mut Commands,
    center: Vec3,
    radius: f32,
    spell_effects: &[(Entity, Vec3, SpellEffectKind)],
    wall_of_fire_query: &Query<&WallOfFireEffect>,
    wall_of_stone_query: &Query<&WallOfStone>,
    spike_growth_query: &Query<&SpikeGrowthZone>,
    grease_query: &Query<(&GreaseZone, Has<GreaseIgnited>)>,
    meteor_fire_query: &Query<&MeteorGroundFire>,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) -> Vec<(Entity, Vec3, SpellEffectKind)> {
    let mut dispelled = Vec::new();

    for &(spell_entity, spell_pos, kind) in spell_effects {
        let edge_dist = spell_edge_distance(
            center,
            spell_entity,
            spell_pos,
            wall_of_fire_query,
            wall_of_stone_query,
            spike_growth_query,
            grease_query,
            meteor_fire_query,
        );

        if edge_dist <= radius {
            dispelled.push((spell_entity, spell_pos, kind));

            despawn_spell_effect(
                commands,
                spell_entity,
                wall_of_stone_query,
                wall_of_fire_query,
                spike_growth_query,
                grease_query,
                meteor_fire_query,
                obstacle_events,
            );
        }
    }

    dispelled
}

/// Computes the XZ distance from a point to the nearest edge of a spell effect's volume.
///
/// For volumetric effects (wall of fire, wall of stone, circular zones), returns the
/// distance to the closest edge of the area rather than the center. Returns 0 if
/// the point is inside the volume. Falls back to center-point distance for unknown types.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spell_edge_distance(
    point: Vec3,
    spell_entity: Entity,
    spell_center: Vec3,
    wall_of_fire_query: &Query<&WallOfFireEffect>,
    wall_of_stone_query: &Query<&WallOfStone>,
    spike_growth_query: &Query<&SpikeGrowthZone>,
    grease_query: &Query<(&GreaseZone, Has<GreaseIgnited>)>,
    meteor_fire_query: &Query<&MeteorGroundFire>,
) -> f32 {
    // Wall of Fire: line segment with half_width
    if let Ok(wall) = wall_of_fire_query.get(spell_entity) {
        let dist_to_line = wall.distance_to_point(point);
        return (dist_to_line - wall.half_width).max(0.0);
    }

    // Wall of Stone: oriented bounding box
    if let Ok(wall) = wall_of_stone_query.get(spell_entity) {
        if wall.contains_point_xz(point) {
            return 0.0;
        }
        let diff = Vec3::new(point.x - wall.center.x, 0.0, point.z - wall.center.z);
        let forward_proj = diff
            .dot(wall.forward)
            .clamp(-wall.half_length, wall.half_length);
        let right_proj = diff
            .dot(wall.right)
            .clamp(-wall.half_width, wall.half_width);
        let closest = wall.center + wall.forward * forward_proj + wall.right * right_proj;
        return xz_distance(point, closest);
    }

    // Spike Growth: circular zone
    if let Ok(zone) = spike_growth_query.get(spell_entity) {
        return (xz_distance(point, zone.origin) - zone.effective_radius()).max(0.0);
    }

    // Grease: circular zone
    if let Ok((zone, _)) = grease_query.get(spell_entity) {
        return (xz_distance(point, zone.origin) - zone.radius).max(0.0);
    }

    // Meteor Ground Fire: circular zone
    if let Ok(fire) = meteor_fire_query.get(spell_entity) {
        return (xz_distance(point, fire.origin) - fire.radius).max(0.0);
    }

    // Fallback: center-point distance
    xz_distance(point, spell_center)
}

/// Despawns a spell effect entity and cleans up its pathfinding obstacle if applicable.
///
/// Wall of Stone is special: instead of instant despawn, it enters the sinking animation
/// so it visually sinks into the ground with dust VFX before being cleaned up.
#[allow(clippy::too_many_arguments)]
pub(crate) fn despawn_spell_effect(
    commands: &mut Commands,
    spell_entity: Entity,
    wall_of_stone_query: &Query<&WallOfStone>,
    wall_of_fire_query: &Query<&WallOfFireEffect>,
    spike_growth_query: &Query<&SpikeGrowthZone>,
    grease_query: &Query<(&GreaseZone, Has<GreaseIgnited>)>,
    meteor_fire_query: &Query<&MeteorGroundFire>,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) {
    use crate::game::multiplayer::components::NetworkedSpellEffect;

    // Wall of Stone -- trigger sink animation instead of instant despawn.
    // The obstacle is removed immediately so units can path through,
    // but the wall entity sinks visually over WALL_SINK_DURATION before cleanup.
    if let Ok(wall) = wall_of_stone_query.get(spell_entity) {
        let obs_bounds = wall.obstacle_bounds();
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::obb_from_center(
                wall.center,
                wall.forward,
                wall.half_length,
                wall.half_width,
            )),
            rebuild: false,
        });

        // Trigger sinking animation — the existing tick/animate/cleanup pipeline
        // will handle the visual sink and eventual despawn.
        let sink_duration =
            crate::game::units::wizard::spells::wall_of_stone::constants::WALL_SINK_DURATION;
        commands.entity(spell_entity).insert(
            crate::game::units::wizard::spells::wall_of_stone::components::DispelledWall {
                sink_duration,
            },
        );
        // Remove the NetworkedSpellEffect so the dispel impact doesn't re-target this wall
        commands
            .entity(spell_entity)
            .remove::<NetworkedSpellEffect>();
        return;
    }

    // Wall of Fire -- hazard obstacle
    if let Ok(effect) = wall_of_fire_query.get(spell_entity) {
        let a = Vec2::new(effect.start.x, effect.start.z);
        let b = Vec2::new(effect.end.x, effect.end.z);
        let dir = b - a;
        let perp = Vec2::new(-dir.y, dir.x).normalize_or_zero() * effect.half_width;
        let c0 = a + perp;
        let c1 = a - perp;
        let c2 = b + perp;
        let c3 = b - perp;
        let min_x = c0.x.min(c1.x).min(c2.x).min(c3.x) - OBSTACLE_BUFFER;
        let max_x = c0.x.max(c1.x).max(c2.x).max(c3.x) + OBSTACLE_BUFFER;
        let min_y = c0.y.min(c1.y).min(c2.y).min(c3.y) - OBSTACLE_BUFFER;
        let max_y = c0.y.max(c1.y).max(c2.y).max(c3.y) + OBSTACLE_BUFFER;

        obstacle_events.write(ObstacleChanged {
            bounds: Rect::new(min_x, min_y, max_x, max_y),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::obb_from_wall(
                effect.start,
                effect.end,
                effect.half_width + OBSTACLE_BUFFER,
            )),
            rebuild: false,
        });
    }

    // Spike Growth -- hazard obstacle (circular zone)
    if let Ok(zone) = spike_growth_query.get(spell_entity) {
        let origin_2d = Vec2::new(zone.origin.x, zone.origin.z);
        let buffered_radius = zone.effective_radius() + OBSTACLE_BUFFER;
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
            rebuild: false,
        });
    }

    // Grease -- hazard obstacle when ignited
    if let Ok((zone, is_ignited)) = grease_query.get(spell_entity)
        && is_ignited
    {
        let origin_2d = Vec2::new(zone.origin.x, zone.origin.z);
        let buffered_radius = zone.radius + OBSTACLE_BUFFER;
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
            rebuild: false,
        });
    }

    // Meteor Ground Fire -- hazard obstacle
    if let Ok(fire) = meteor_fire_query.get(spell_entity) {
        let origin_2d = Vec2::new(fire.origin.x, fire.origin.z);
        let buffered = fire.radius + OBSTACLE_BUFFER;
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0)),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::circle(origin_2d, buffered)),
            rebuild: false,
        });
    }

    commands.entity(spell_entity).try_despawn();
}

/// Removes `MindControlled` from all units within `radius` of `center`.
/// Returns the number of mind-controlled units that were freed.
pub(crate) fn remove_mind_control_in_radius(
    commands: &mut Commands,
    center: Vec3,
    radius: f32,
    mind_controlled_iter: impl Iterator<Item = (Entity, Vec3)>,
) -> u32 {
    let mut count = 0;
    for (entity, position) in mind_controlled_iter {
        if xz_distance(position, center) <= radius {
            commands.entity(entity).remove::<MindControlled>();
            count += 1;
        }
    }
    count
}

/// Removes `SpellShield` and `ShielderDamageReduction` from all units within
/// `radius` of `center`. Returns the number of shields stripped. The king's
/// always-on aura visual is independent of the shield, so nothing needs to
/// despawn here.
pub(crate) fn strip_spell_shields_in_radius(
    commands: &mut Commands,
    center: Vec3,
    radius: f32,
    shielded_units: impl Iterator<Item = (Entity, Vec3)>,
) -> u32 {
    let mut count = 0;
    for (entity, position) in shielded_units {
        if xz_distance(position, center) <= radius {
            commands.entity(entity).remove::<SpellShield>();
            commands.entity(entity).remove::<ShielderDamageReduction>();
            count += 1;
        }
    }
    count
}
