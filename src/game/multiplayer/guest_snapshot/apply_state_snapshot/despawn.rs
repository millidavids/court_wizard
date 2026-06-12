use std::collections::HashSet;

use bevy::prelude::*;

use crate::networking::entity_map::NetworkEntityMap;

/// Despawns ghost entities whose IDs are no longer present in the latest
/// snapshot's `seen_ids` set, and removes them from `entity_map`.
pub(super) fn despawn_stale_ghosts(
    commands: &mut Commands,
    entity_map: &mut ResMut<NetworkEntityMap>,
    seen_ids: &HashSet<u32>,
) {
    let stale_ids: Vec<u32> = entity_map
        .remote_to_local
        .keys()
        .copied()
        .filter(|id| !seen_ids.contains(id))
        .collect();

    for stale_id in stale_ids {
        if let Some(entity) = entity_map.remove_by_remote(stale_id)
            && let Ok(mut entity_commands) = commands.get_entity(entity)
        {
            entity_commands.try_despawn();
        }
    }
}
