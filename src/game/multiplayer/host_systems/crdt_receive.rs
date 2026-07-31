use bevy::prelude::*;

use crate::game::units::components::{
    Health, RemoteElectricEffect, RemoteFireEffect, RemoteFrostEffect,
};
use crate::networking::crdt::CrdtHealth;
use crate::networking::entity_map::NetworkEntityId;
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{CrdtSnapshot, UNRELIABLE_CRDT_SNAPSHOT, UnitFlags};

/// Receives CRDT health updates from the guest and merges into local unit state.
///
/// The guest sends its CRDT counters (guest spell damage/healing) over the
/// unreliable channel. The host merges these using element-wise max so that
/// guest spell effects are reflected in the host's simulation. Critically
/// for healing: the CRDT path lets the guest cast a heal spell (which
/// touches `Health.current` upward), have that propagate to the host's
/// authoritative unit, and have the host's HP converge — without needing
/// per-spell heal messages.
pub fn receive_crdt_snapshot(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    mut units: Query<(
        Entity,
        &NetworkEntityId,
        &mut CrdtHealth,
        &mut Health,
        Has<RemoteFireEffect>,
        Has<RemoteFrostEffect>,
        Has<RemoteElectricEffect>,
    )>,
) {
    let raw_data: Vec<Vec<u8>> = connection.incoming_unreliable.drain(..).collect();
    let mut other_data = Vec::new();
    let mut latest_crdt_data: Option<&[u8]> = None;

    for data in &raw_data {
        if data.is_empty() {
            continue;
        }
        match data[0] {
            UNRELIABLE_CRDT_SNAPSHOT => {
                latest_crdt_data = Some(&data[1..]);
            }
            _ => {
                other_data.push(data.clone());
            }
        }
    }

    connection.incoming_unreliable = other_data;

    let Some(crdt_bytes) = latest_crdt_data else {
        return;
    };

    let Ok(snapshot) = bincode::deserialize::<CrdtSnapshot>(crdt_bytes) else {
        warn!(
            "Failed to deserialize CRDT snapshot ({} bytes)",
            crdt_bytes.len()
        );
        return;
    };

    let mut update_map = std::collections::HashMap::with_capacity(snapshot.units.len());
    for (i, update) in snapshot.units.iter().enumerate() {
        update_map.insert(update.id, i);
    }

    for (
        entity,
        net_id,
        mut crdt_health,
        mut health,
        has_remote_fire,
        has_remote_frost,
        has_remote_electric,
    ) in &mut units
    {
        if let Some(&idx) = update_map.get(&net_id.0) {
            let update = &snapshot.units[idx];
            let remote = CrdtHealth {
                max_hp: crdt_health.max_hp,
                damage: update.damage,
                healing: update.healing,
            };
            crdt_health.merge(&remote);
            health.current = crdt_health.current_hp();

            // CRDT effects slot is u8 (lower 8 bits) — cast u32 constants to u8.
            let remote_fire = update.effects & (UnitFlags::FIRE_EFFECT as u8) != 0;
            let remote_frost = update.effects & (UnitFlags::FROST_EFFECT as u8) != 0;
            let remote_electric = update.effects & (UnitFlags::ELECTRIC_EFFECT as u8) != 0;

            if remote_fire && !has_remote_fire {
                commands.entity(entity).insert(RemoteFireEffect);
            } else if !remote_fire && has_remote_fire {
                commands.entity(entity).remove::<RemoteFireEffect>();
            }
            if remote_frost && !has_remote_frost {
                commands.entity(entity).insert(RemoteFrostEffect);
            } else if !remote_frost && has_remote_frost {
                commands.entity(entity).remove::<RemoteFrostEffect>();
            }
            if remote_electric && !has_remote_electric {
                commands.entity(entity).insert(RemoteElectricEffect);
            } else if !remote_electric && has_remote_electric {
                commands.entity(entity).remove::<RemoteElectricEffect>();
            }
        }
    }
}
