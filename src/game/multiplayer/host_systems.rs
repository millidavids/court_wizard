//! Host-only multiplayer systems.
//!
//! These systems run on the host to assign network IDs to entities and
//! send state snapshots to the guest every frame.

use bevy::prelude::*;

use crate::game::units::archer::Archer;
use crate::game::units::components::{Corpse, Health, KingsGuard, Team};
use crate::game::units::king::components::King;
use crate::networking::entity_map::{EntityIdCounter, NetworkEntityId};
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{GameSnapshot, SnapshotTick, build_unit_snapshot};

/// Assigns `NetworkEntityId` to newly spawned entities that have `Health` + `Team`
/// but don't yet have a network ID.
pub fn assign_network_ids(
    mut commands: Commands,
    mut counter: ResMut<EntityIdCounter>,
    new_units: Query<Entity, (With<Health>, With<Team>, Without<NetworkEntityId>)>,
) {
    for entity in &new_units {
        let net_id = counter.next();
        commands.entity(entity).insert(net_id);
    }
}

/// Serializes the full game state and sends it over the unreliable channel.
///
/// Runs every frame (~60Hz). Queries all entities with a `NetworkEntityId` and
/// builds a compact `GameSnapshot` serialized with `bincode`.
#[allow(clippy::type_complexity)]
pub fn send_state_snapshots(
    mut tick: ResMut<SnapshotTick>,
    mut connection: ResMut<NetworkConnection>,
    units: Query<(
        &NetworkEntityId,
        &Transform,
        &Team,
        &Health,
        Has<Corpse>,
        Has<King>,
        Has<Archer>,
        Has<KingsGuard>,
    )>,
) {
    tick.0 = tick.0.wrapping_add(1);

    let mut snapshot = GameSnapshot {
        tick: tick.0,
        units: Vec::with_capacity(units.iter().len()),
    };

    for (net_id, transform, team, health, is_corpse, is_king, is_archer, is_guard) in &units {
        snapshot.units.push(build_unit_snapshot(
            net_id, transform, team, health, is_corpse, is_king, is_archer, is_guard,
        ));
    }

    if let Ok(data) = bincode::serialize(&snapshot) {
        connection.outgoing_unreliable.push(data);
    }
}
