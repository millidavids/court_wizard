//! Guest-only multiplayer systems.
//!
//! The guest receives state snapshots from the host and renders ghost entities
//! that mirror the host's game state. Ghost entities are lightweight visual
//! representations with no simulation components.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::game::components::Billboard;
use crate::game::resources::GameOutcome;
use crate::game::units::archer::ArcherAssets;
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::king::resources::KingAssets;
use crate::networking::entity_map::NetworkEntityMap;
use crate::networking::protocol::{GameOverResult, NetworkMessage};
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{GameSnapshot, UnitFlags, u8_to_team};
use crate::state::MultiplayerGameState;

use super::components::{GhostEntity, OnMultiplayerGameScreen};

/// Receives the latest state snapshot from the host and creates/updates/despawns
/// ghost entities to match the host's game state.
pub fn apply_state_snapshot(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    mut entity_map: ResMut<NetworkEntityMap>,
    infantry_assets: Res<InfantryAssets>,
    archer_assets: Res<ArcherAssets>,
    king_assets: Res<KingAssets>,
    mut ghost_query: Query<(&mut Transform, &mut MeshMaterial3d<StandardMaterial>), With<GhostEntity>>,
) {
    // Take only the latest snapshot (discard stale ones)
    let raw_snapshots: Vec<Vec<u8>> = connection.incoming_unreliable.drain(..).collect();
    let Some(latest_data) = raw_snapshots.last() else {
        return;
    };

    let Ok(snapshot) = bincode::deserialize::<GameSnapshot>(latest_data) else {
        return;
    };

    // Track which IDs are present in this snapshot
    let mut seen_ids = HashSet::with_capacity(snapshot.units.len());

    for unit in &snapshot.units {
        seen_ids.insert(unit.id);

        let is_corpse = unit.flags & UnitFlags::CORPSE != 0;
        let is_king = unit.flags & UnitFlags::KING != 0;
        let is_archer = unit.flags & UnitFlags::ARCHER != 0;
        let is_guard = unit.flags & UnitFlags::KINGS_GUARD != 0;
        let team = u8_to_team(unit.team);

        // Pick the correct preloaded material based on unit type, team, and alive/dead
        let material_handle = pick_material(
            &infantry_assets,
            &archer_assets,
            &king_assets,
            team,
            is_corpse,
            is_king,
            is_archer,
            is_guard,
        );

        // Pick the correct preloaded mesh based on unit type
        let mesh_handle = if is_king {
            king_assets.mesh.clone()
        } else if is_archer {
            archer_assets.mesh.clone()
        } else {
            // Infantry and King's Guard both use infantry mesh
            infantry_assets.mesh.clone()
        };

        let pos = Vec3::new(unit.x, unit.y, unit.z);

        if let Some(&local_entity) = entity_map.remote_to_local.get(&unit.id) {
            // Update existing ghost entity
            if let Ok((mut transform, material_ref)) = ghost_query.get_mut(local_entity) {
                transform.translation = pos;

                // Update material if it changed (e.g., unit became corpse)
                if material_ref.0 != material_handle {
                    commands.entity(local_entity).insert(MeshMaterial3d(material_handle));
                }
            }
        } else {
            // Spawn new ghost entity using preloaded Circle meshes
            let entity = commands
                .spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material_handle),
                    Transform::from_translation(pos),
                    Billboard,
                    GhostEntity,
                    OnMultiplayerGameScreen,
                ))
                .id();

            entity_map.insert(unit.id, entity);
        }
    }

    // Despawn ghost entities whose IDs are no longer in the snapshot
    let stale_ids: Vec<u32> = entity_map
        .remote_to_local
        .keys()
        .copied()
        .filter(|id| !seen_ids.contains(id))
        .collect();

    for stale_id in stale_ids {
        if let Some(entity) = entity_map.remove_by_remote(stale_id) {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.despawn();
            }
        }
    }
}

/// Picks the correct preloaded material handle for a ghost entity.
///
/// Corpse materials already have low alpha baked into the preloaded assets.
fn pick_material(
    infantry_assets: &InfantryAssets,
    archer_assets: &ArcherAssets,
    king_assets: &KingAssets,
    team: crate::game::units::components::Team,
    is_corpse: bool,
    is_king: bool,
    is_archer: bool,
    is_guard: bool,
) -> Handle<StandardMaterial> {
    use crate::game::units::components::Team;

    if is_corpse {
        if is_king {
            king_assets.corpse_material.clone()
        } else if is_archer {
            match team {
                Team::Defenders => archer_assets.defender_corpse_material.clone(),
                Team::Attackers => archer_assets.attacker_corpse_material.clone(),
                Team::Undead => archer_assets.undead_corpse_material.clone(),
            }
        } else {
            // Infantry and King's Guard corpses use the same material
            match team {
                Team::Defenders => infantry_assets.defender_corpse_material.clone(),
                Team::Attackers => infantry_assets.attacker_corpse_material.clone(),
                Team::Undead => infantry_assets.undead_corpse_material.clone(),
            }
        }
    } else if is_king {
        king_assets.material.clone()
    } else if is_archer {
        match team {
            Team::Defenders => archer_assets.defender_material.clone(),
            Team::Attackers => archer_assets.attacker_material.clone(),
            Team::Undead => archer_assets.undead_material.clone(),
        }
    } else if is_guard {
        infantry_assets.kings_guard_material.clone()
    } else {
        // Infantry
        match team {
            Team::Defenders => infantry_assets.defender_material.clone(),
            Team::Attackers => infantry_assets.attacker_material.clone(),
            Team::Undead => infantry_assets.undead_material.clone(),
        }
    }
}

/// Listens for `GameOver` messages from the host and transitions to the score screen.
pub fn handle_game_over_message(
    mut connection: ResMut<NetworkConnection>,
    mut game_outcome: ResMut<GameOutcome>,
    mut next_state: ResMut<NextState<MultiplayerGameState>>,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }

    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::GameOver(result) => {
                *game_outcome = match result {
                    GameOverResult::HostWins => GameOutcome::DefeatKingDied,
                    GameOverResult::GuestWins => GameOutcome::Victory,
                };
                next_state.set(MultiplayerGameState::ScoreScreen);
            }
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}
