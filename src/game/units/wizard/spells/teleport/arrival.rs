//! Teleport execution: scatter, swap, recall, unit teleport.

use bevy::prelude::*;
use rand::Rng;

use super::super::super::components::{CastingState, LocalWizard, PrimedSpell, Spell};
use super::components::{
    DimensionalRift, DisorientingHaste, LingeringGateMarker, RiftCooldown, TeleportCaster,
    TeleportDestinationCircle, TeleportSourceCircle, TeleportTalentParams,
};
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::BATTLEFIELD_SIZE;
use crate::game::units::DamageType;
use crate::game::units::components::{Airborne, Corpse, Stunned, Team, Teleportable};
use crate::game::units::wizard::spells::utils::xz_distance;

/// Computes talent parameters from active talent selections.
pub(super) fn execute_teleport(
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

/// Applies post-teleport talent effects (stun, haste, dimensional rift).
/// Effects are applied directly to the teleported entities rather than querying by position,
/// since teleport uses deferred commands (transforms haven't been applied yet).
/// Returns the rift entity if Dimensional Rift was spawned.
pub(super) fn apply_post_teleport_effects(
    commands: &mut Commands,
    talent_params: &TeleportTalentParams,
    source_pos: Vec3,
    dest_pos: Vec3,
    teleported_entities: &[Entity],
) -> Option<Entity> {
    // Disorienting Arrival: stun AND haste all teleported units
    if talent_params.disorienting_arrival {
        for &entity in teleported_entities {
            commands
                .entity(entity)
                .insert(Stunned::new(DISORIENTING_STUN_DURATION));
            commands.entity(entity).insert(DisorientingHaste::new(
                DISORIENTING_ATTACK_SPEED,
                DISORIENTING_ATTACK_SPEED_DURATION,
            ));
        }
    }

    // Dimensional Rift: spawn persistent two-way portal
    if talent_params.dimensional_rift {
        let rift_entity = commands
            .spawn((
                DimensionalRift {
                    source_pos,
                    dest_pos,
                    walk_radius: DIMENSIONAL_RIFT_WALK_RADIUS,
                    time_remaining: DIMENSIONAL_RIFT_DURATION,
                    two_way: talent_params.swap_mode,
                },
                OnGameplayScreen,
            ))
            .id();
        return Some(rift_entity);
    }

    None
}

/// Ticks Dimensional Rift portals and teleports units that walk through them.
pub fn tick_dimensional_rift(
    mut commands: Commands,
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut rifts: Query<(Entity, &mut DimensionalRift)>,
    mut units: Query<
        (Entity, &mut Transform, Option<&RiftCooldown>),
        (With<Teleportable>, Without<Corpse>),
    >,
) {
    let delta = time.delta_secs();

    for (rift_entity, mut rift) in rifts.iter_mut() {
        rift.time_remaining -= delta;
        if rift.time_remaining <= 0.0 {
            commands.entity(rift_entity).try_despawn();
            continue;
        }

        let rng = &mut game_rng.0;

        for (unit_entity, mut transform, cooldown) in units.iter_mut() {
            // Skip units on cooldown from recent rift teleport
            if cooldown.is_some() {
                continue;
            }

            let pos = transform.translation;

            if xz_distance(pos, rift.source_pos) <= rift.walk_radius {
                // Near source portal → teleport to destination
                let new_pos =
                    random_position_in_circle(rng, rift.dest_pos, rift.walk_radius, pos.y);
                transform.translation = new_pos;
                commands.entity(unit_entity).insert(RiftCooldown {
                    time_remaining: DIMENSIONAL_RIFT_UNIT_COOLDOWN,
                });
            } else if rift.two_way && xz_distance(pos, rift.dest_pos) <= rift.walk_radius {
                // Near destination portal → teleport to source (only with Swap talent)
                let new_pos =
                    random_position_in_circle(rng, rift.source_pos, rift.walk_radius, pos.y);
                transform.translation = new_pos;
                commands.entity(unit_entity).insert(RiftCooldown {
                    time_remaining: DIMENSIONAL_RIFT_UNIT_COOLDOWN,
                });
            }
        }
    }
}

/// Ticks Lingering Gate markers and removes expired ones.
pub fn tick_lingering_gate(
    mut commands: Commands,
    time: Res<Time>,
    mut gates: Query<(Entity, &mut LingeringGateMarker)>,
    mut caster_query: Query<&mut TeleportCaster, With<LocalWizard>>,
) {
    let delta = time.delta_secs();

    for (gate_entity, mut gate) in gates.iter_mut() {
        gate.time_remaining -= delta;
        if gate.time_remaining <= 0.0 {
            commands.entity(gate_entity).try_despawn();

            // Reset caster state when gate expires
            if let Ok(mut caster) = caster_query.single_mut()
                && caster.destination_circle == Some(gate_entity)
            {
                caster.destination_circle = None;
                caster.destination_position = None;
                caster.lingering_gate_active = false;
            }
        }
    }
}

/// Updates pulse animations for both destination and source circles.
pub fn update_circle_animations(
    time: Res<Time>,
    mut destination_query: Query<
        (&mut Transform, &mut TeleportDestinationCircle),
        Without<TeleportSourceCircle>,
    >,
    mut source_query: Query<(&mut Transform, &mut TeleportSourceCircle)>,
) {
    // Update destination circles
    for (mut transform, mut indicator) in &mut destination_query {
        indicator.time_alive += time.delta_secs();

        // Only apply pulse animation after growth is mostly complete
        if transform.scale.x >= indicator.base_radius * PULSE_THRESHOLD {
            let pulse = indicator.pulse_scale();
            transform.scale = Vec3::splat(indicator.base_radius * pulse);
        }
    }

    // Update source circles
    for (mut transform, mut indicator) in &mut source_query {
        indicator.time_alive += time.delta_secs();

        let radius = CIRCLE_RADIUS * indicator.empowerment;
        // Only apply pulse animation after growth is mostly complete
        if transform.scale.x >= radius * PULSE_THRESHOLD {
            let pulse = indicator.pulse_scale();
            transform.scale = Vec3::splat(radius * pulse);
        }
    }
}

/// Generates a random position within a circle, clamped to battlefield bounds.
fn random_position_in_circle(rng: &mut impl Rng, center: Vec3, radius: f32, y: f32) -> Vec3 {
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let random_radius = rng.random_range(0.0..radius);

    let new_x = (center.x + angle.cos() * random_radius)
        .clamp(-BATTLEFIELD_SIZE / 2.0, BATTLEFIELD_SIZE / 2.0);
    let new_z = (center.z + angle.sin() * random_radius)
        .clamp(-BATTLEFIELD_SIZE / 2.0, BATTLEFIELD_SIZE / 2.0);

    Vec3::new(new_x, y, new_z)
}

/// Cleans up teleport circles and caster state when the player switches away
/// from the Teleport spell while a teleport is in progress.
pub fn cleanup_teleport_on_spell_switch(
    mut commands: Commands,
    mut wizard_query: Query<
        (&PrimedSpell, &mut CastingState),
        (With<LocalWizard>, Changed<PrimedSpell>),
    >,
    mut caster_query: Query<&mut TeleportCaster, With<LocalWizard>>,
) {
    let Ok((primed_spell, mut casting_state)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell == Spell::Teleport {
        return;
    }

    let Ok(mut caster) = caster_query.single_mut() else {
        return;
    };

    // Only clean up if there's actually an in-progress teleport (circles exist).
    // TeleportCaster persists on the wizard, so we must check for active state.
    let has_circles = caster.destination_circle.is_some() || caster.source_circle.is_some();
    if !has_circles {
        return;
    }

    if let Some(dest_entity) = caster.destination_circle {
        commands.entity(dest_entity).try_despawn();
    }
    if let Some(source_entity) = caster.source_circle {
        commands.entity(source_entity).try_despawn();
    }
    caster.destination_circle = None;
    caster.destination_position = None;
    caster.source_circle = None;
    caster.lingering_gate_active = false;
    if !matches!(*casting_state, CastingState::Resting) {
        casting_state.cancel();
    }
}
