use bevy::prelude::*;
use rand::Rng;

use super::super::components::{
    TeleportDestinationCircle, TeleportSourceCircle, TeleportTalentParams,
};
use super::super::constants::*;
use crate::game::constants::BATTLEFIELD_SIZE;
use crate::game::units::DamageType;
use crate::game::units::components::{Airborne, Corpse, Team, Teleportable};
use crate::game::units::wizard::spells::utils::xz_distance;

/// Computes talent parameters from active talent selections.
pub(crate) fn execute_teleport(
    rng: &mut impl Rng,
    source_center: Vec3,
    dest_center: Vec3,
    radius: f32,
    units_query: &Query<
        (Entity, &Transform, Option<&Team>),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
            Without<Corpse>,
        ),
    >,
    commands: &mut Commands,
    talent_params: &TeleportTalentParams,
) -> Vec<Entity> {
    if talent_params.teleport_up {
        teleport_units_up(source_center, radius, units_query, commands)
    } else if talent_params.scatterport {
        scatter_enemies(rng, source_center, radius, units_query, commands)
    } else if talent_params.swap_mode {
        swap_units(
            rng,
            source_center,
            dest_center,
            radius,
            units_query,
            commands,
        )
    } else if talent_params.emergency_recall {
        recall_allies(
            rng,
            source_center,
            dest_center,
            radius,
            units_query,
            commands,
        )
    } else {
        teleport_units_with_radius(
            rng,
            source_center,
            dest_center,
            radius,
            units_query,
            commands,
        )
    }
}

/// Up: teleports all units within radius straight up into the air.
/// They fall back down and take fall damage on landing.
fn teleport_units_up(
    center: Vec3,
    radius: f32,
    units_query: &Query<
        (Entity, &Transform, Option<&Team>),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
            Without<Corpse>,
        ),
    >,
    commands: &mut Commands,
) -> Vec<Entity> {
    let mut teleported = Vec::new();

    for (entity, transform, _team) in units_query.iter() {
        if xz_distance(transform.translation, center) <= radius {
            // Place the unit high in the air with zero velocity — it falls from there.
            // The airborne system handles visual Y offset via base_y + height.
            commands.entity(entity).insert(Airborne {
                vertical_velocity: 0.0,
                height: TELEPORT_UP_HEIGHT,
                base_y: transform.translation.y,
                gravity: TELEPORT_UP_GRAVITY,
                damage_type: DamageType::Force,
            });
            teleported.push(entity);
        }
    }

    teleported
}

/// Teleports all units within a specified radius of the source center to random positions
/// within the same radius of the destination center.
/// Returns the list of teleported entity IDs.
pub(crate) fn teleport_units_with_radius(
    rng: &mut impl Rng,
    source_center: Vec3,
    dest_center: Vec3,
    radius: f32,
    units_query: &Query<
        (Entity, &Transform, Option<&Team>),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
            Without<Corpse>,
        ),
    >,
    commands: &mut Commands,
) -> Vec<Entity> {
    let mut teleported = Vec::new();

    for (entity, transform, _team) in units_query.iter() {
        if xz_distance(transform.translation, source_center) <= radius {
            let new_position =
                random_position_in_circle(rng, dest_center, radius, transform.translation.y);
            let mut new_transform = *transform;
            new_transform.translation = new_position;
            commands.entity(entity).insert(new_transform);
            teleported.push(entity);
        }
    }

    teleported
}

/// Scatterport talent: scatters all units to random locations in a large radius.
fn scatter_enemies(
    rng: &mut impl Rng,
    source_center: Vec3,
    radius: f32,
    units_query: &Query<
        (Entity, &Transform, Option<&Team>),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
            Without<Corpse>,
        ),
    >,
    commands: &mut Commands,
) -> Vec<Entity> {
    let mut teleported = Vec::new();

    for (entity, transform, _team) in units_query.iter() {
        if xz_distance(transform.translation, source_center) <= radius {
            let new_position = random_position_in_circle(
                rng,
                source_center,
                SCATTERPORT_RADIUS,
                transform.translation.y,
            );
            let mut new_transform = *transform;
            new_transform.translation = new_position;
            commands.entity(entity).insert(new_transform);
            teleported.push(entity);
        }
    }

    teleported
}

/// Swap talent: swaps all units between two circles simultaneously.
fn swap_units(
    rng: &mut impl Rng,
    source_center: Vec3,
    dest_center: Vec3,
    radius: f32,
    units_query: &Query<
        (Entity, &Transform, Option<&Team>),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
            Without<Corpse>,
        ),
    >,
    commands: &mut Commands,
) -> Vec<Entity> {
    let mut teleported = Vec::new();

    // Collect units in each circle first (can't modify while iterating)
    let mut source_units: Vec<(Entity, Vec3)> = Vec::new();
    let mut dest_units: Vec<(Entity, Vec3)> = Vec::new();

    for (entity, transform, _team) in units_query.iter() {
        if xz_distance(transform.translation, source_center) <= radius {
            source_units.push((entity, transform.translation));
        } else if xz_distance(transform.translation, dest_center) <= radius {
            dest_units.push((entity, transform.translation));
        }
    }

    // Move source units to destination
    for (entity, original_pos) in &source_units {
        let new_position = random_position_in_circle(rng, dest_center, radius, original_pos.y);
        commands
            .entity(*entity)
            .insert(Transform::from_translation(new_position));
        teleported.push(*entity);
    }

    // Move destination units to source
    for (entity, original_pos) in &dest_units {
        let new_position = random_position_in_circle(rng, source_center, radius, original_pos.y);
        commands
            .entity(*entity)
            .insert(Transform::from_translation(new_position));
        teleported.push(*entity);
    }

    teleported
}

/// Emergency Recall talent: teleports only allied (Defender) units to the King's spawn position.
fn recall_allies(
    rng: &mut impl Rng,
    source_center: Vec3,
    dest_center: Vec3,
    radius: f32,
    units_query: &Query<
        (Entity, &Transform, Option<&Team>),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
            Without<Corpse>,
        ),
    >,
    commands: &mut Commands,
) -> Vec<Entity> {
    let mut teleported = Vec::new();

    for (entity, transform, team) in units_query.iter() {
        // Only recall defenders (skip entities without a team, like rocks)
        if team != Some(&Team::Defenders) {
            continue;
        }

        if xz_distance(transform.translation, source_center) <= radius {
            let new_position =
                random_position_in_circle(rng, dest_center, radius, transform.translation.y);
            let mut new_transform = *transform;
            new_transform.translation = new_position;
            commands.entity(entity).insert(new_transform);
            teleported.push(entity);
        }
    }

    teleported
}

/// Generates a random position within a circle, clamped to battlefield bounds.
pub(crate) fn random_position_in_circle(
    rng: &mut impl Rng,
    center: Vec3,
    radius: f32,
    y: f32,
) -> Vec3 {
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let random_radius = rng.random_range(0.0..radius);

    let new_x = (center.x + angle.cos() * random_radius)
        .clamp(-BATTLEFIELD_SIZE / 2.0, BATTLEFIELD_SIZE / 2.0);
    let new_z = (center.z + angle.sin() * random_radius)
        .clamp(-BATTLEFIELD_SIZE / 2.0, BATTLEFIELD_SIZE / 2.0);

    Vec3::new(new_x, y, new_z)
}
