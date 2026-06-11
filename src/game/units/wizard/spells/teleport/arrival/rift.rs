use bevy::prelude::*;

use super::super::components::{
    DimensionalRift, LingeringGateMarker, RiftCooldown, TeleportCaster,
};
use super::super::constants::*;
use super::teleport_logic::random_position_in_circle;
use crate::game::units::components::{Corpse, Teleportable};
use crate::game::units::wizard::components::LocalWizard;
use crate::game::units::wizard::spells::utils::xz_distance;

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
