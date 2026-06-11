use bevy::prelude::*;

use crate::game::units::components::{FireDoT, FrostAccumulation, Shocked};
use crate::networking::crdt::CrdtHealth;
use crate::networking::entity_map::NetworkEntityId;
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{
    CrdtSnapshot, CrdtUnitUpdate, UNRELIABLE_CRDT_SNAPSHOT, UnitFlags,
};

use super::super::components::GhostEntity;

/// Sends the guest's local CRDT health state to the host for merging.
///
/// Collects CrdtHealth from all ghost entities and sends as a compact
/// CrdtSnapshot over the unreliable channel. The host merges these
/// counters into its local unit state.
pub fn send_crdt_snapshot(
    mut connection: ResMut<NetworkConnection>,
    crdt_units: Query<
        (
            &NetworkEntityId,
            &CrdtHealth,
            Has<FireDoT>,
            Has<FrostAccumulation>,
            Has<Shocked>,
        ),
        With<GhostEntity>,
    >,
) {
    let mut snapshot = CrdtSnapshot {
        units: Vec::with_capacity(crdt_units.iter().len()),
    };

    for (net_id, crdt, has_fire, has_frost, has_electric) in &crdt_units {
        // CRDT effects slot is u8 — UnitFlags constants are u16 now (after
        // COMBAT_ANIMATION moved them past the 8-bit window), so cast
        // each one back. All three fire/frost/electric flags live in the
        // low byte so the truncation is lossless.
        let mut effects = 0u8;
        if has_fire {
            effects |= UnitFlags::FIRE_EFFECT as u8;
        }
        if has_frost {
            effects |= UnitFlags::FROST_EFFECT as u8;
        }
        if has_electric {
            effects |= UnitFlags::ELECTRIC_EFFECT as u8;
        }
        snapshot.units.push(CrdtUnitUpdate {
            id: net_id.0,
            damage: crdt.damage,
            healing: crdt.healing,
            effects,
        });
    }

    if let Ok(data) = bincode::serialize(&snapshot) {
        let mut prefixed = Vec::with_capacity(1 + data.len());
        prefixed.push(UNRELIABLE_CRDT_SNAPSHOT);
        prefixed.extend_from_slice(&data);
        connection.outgoing_unreliable.push(prefixed);
    }
}
