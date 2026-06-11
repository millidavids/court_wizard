use bevy::prelude::*;

use crate::game::multiplayer::components::{NetworkedSpellEffect, NoSnapshot};
use crate::game::units::components::{Health, Team};
use crate::networking::entity_map::{EntityIdCounter, NetworkEntityId};

/// Assigns `NetworkEntityId` to newly spawned entities that have `Health` + `Team`
/// or `NetworkedSpellEffect` but don't yet have a network ID.
pub fn assign_network_ids(
    mut commands: Commands,
    mut counter: ResMut<EntityIdCounter>,
    // Wizards are spawned locally on BOTH peers (never ghosted), so they carry
    // `NoSnapshot` (added in their spawn bundle) to stay out of the snapshot
    // stream. Versus wizards lack `Team` and were already skipped; the co-op
    // host's wizard HAS `Team::Defenders`, so `NoSnapshot` is what keeps both
    // co-op wizards from being double-rendered as ghosts.
    new_units: Query<
        Entity,
        (
            With<Health>,
            With<Team>,
            Without<NetworkEntityId>,
            Without<NoSnapshot>,
        ),
    >,
    new_effects: Query<Entity, (With<NetworkedSpellEffect>, Without<NetworkEntityId>)>,
) {
    for entity in new_units.iter().chain(new_effects.iter()) {
        let net_id = counter.next();
        commands.entity(entity).insert(net_id);
    }
}
