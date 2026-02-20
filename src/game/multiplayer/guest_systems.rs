//! Guest-only multiplayer systems.
//!
//! The guest receives state snapshots from the host and renders ghost entities
//! that mirror the host's game state. Ghost entities are lightweight visual
//! representations with no simulation components.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::game::components::Billboard;
use crate::game::constants::{ATTACKER_BASE, DEFENDER_BASE};
use crate::game::units::archer::styles::ARCHER_RADIUS;
use crate::game::units::infantry::styles::UNIT_RADIUS;
use crate::game::units::king::constants::KING_RADIUS;
use crate::networking::entity_map::NetworkEntityMap;
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{GameSnapshot, UnitFlags, u8_to_team};

use super::components::{GhostEntity, OnMultiplayerGameScreen};

/// Color for corpse ghost entities.
const CORPSE_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);

/// Color for undead ghost entities.
const UNDEAD_COLOR: Color = Color::srgb(0.4, 0.55, 0.4);

/// Receives the latest state snapshot from the host and creates/updates/despawns
/// ghost entities to match the host's game state.
pub fn apply_state_snapshot(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut connection: ResMut<NetworkConnection>,
    mut entity_map: ResMut<NetworkEntityMap>,
    mut ghost_query: Query<(&mut Transform, &mut MeshMaterial3d<StandardMaterial>), With<GhostEntity>>,
    material_assets: Res<Assets<StandardMaterial>>,
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

        // Determine visual properties
        let color = if is_corpse {
            CORPSE_COLOR
        } else {
            match team {
                crate::game::units::components::Team::Defenders => DEFENDER_BASE,
                crate::game::units::components::Team::Attackers => ATTACKER_BASE,
                crate::game::units::components::Team::Undead => UNDEAD_COLOR,
            }
        };

        let radius = if is_king {
            KING_RADIUS
        } else if is_guard {
            UNIT_RADIUS * 1.2
        } else if is_archer {
            ARCHER_RADIUS
        } else {
            UNIT_RADIUS
        };

        let pos = Vec3::new(unit.x, 0.0, unit.z);

        if let Some(&local_entity) = entity_map.remote_to_local.get(&unit.id) {
            // Update existing ghost entity
            if let Ok((mut transform, material_handle)) = ghost_query.get_mut(local_entity) {
                transform.translation = pos;

                // Update color if it changed (e.g., unit became corpse)
                if let Some(material) = material_assets.get(&material_handle.0) {
                    if material.base_color != color {
                        let new_material = materials.add(StandardMaterial {
                            base_color: color,
                            unlit: true,
                            ..default()
                        });
                        commands.entity(local_entity).insert(MeshMaterial3d(new_material));
                    }
                }
            }
        } else {
            // Spawn new ghost entity
            let hitbox_width = radius * 2.0;
            let hitbox_height = if is_king { 35.0 } else { 25.0 };
            let mesh = Rectangle::new(hitbox_width, hitbox_height);

            let entity = commands
                .spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: color,
                        unlit: true,
                        ..default()
                    })),
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
