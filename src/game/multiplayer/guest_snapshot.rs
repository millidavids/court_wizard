//! Guest snapshot application and CRDT sync.

use super::guest_visuals::pick_material;
use std::collections::HashSet;

use bevy::prelude::*;

use crate::game::components::Billboard;
use crate::game::units::archer::ArcherAssets;
use crate::game::units::components::OriginalMaterial;
use crate::game::units::components::{
    Corpse, FireDoT, FrostAccumulation, Health, RemoteElectricEffect, RemoteFireEffect,
    RemoteFrostEffect, Shocked,
};
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::king::components::{SpellShield, SpellShieldVisual};
use crate::game::units::king::resources::KingAssets;
use crate::networking::crdt::CrdtHealth;
use crate::networking::entity_map::{NetworkEntityId, NetworkEntityMap};
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{
    CrdtSnapshot, CrdtUnitUpdate, GameSnapshot, UNRELIABLE_CRDT_SNAPSHOT, UNRELIABLE_GAME_SNAPSHOT,
    UnitFlags, u8_to_team,
};

use super::components::{GhostArrow, GhostEntity, OnMultiplayerGameScreen};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Receives the latest unit state snapshot from the host and creates/updates/despawns
/// ghost entities to match the host's game state.
///
/// Filters incoming unreliable data by type prefix byte, processing only game
/// snapshots (unit data). Spell visual snapshots are handled by `spell_sync.rs`.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn apply_state_snapshot(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    mut entity_map: ResMut<NetworkEntityMap>,
    infantry_assets: Res<InfantryAssets>,
    archer_assets: Res<ArcherAssets>,
    king_assets: Res<KingAssets>,
    spell_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ghost_query: Query<
        (
            Entity,
            &mut Transform,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut CrdtHealth,
            &mut Health,
            Option<&OriginalMaterial>,
            Has<RemoteFireEffect>,
            Has<RemoteFrostEffect>,
            Has<RemoteElectricEffect>,
            Has<SpellShield>,
            Has<Corpse>,
        ),
        With<GhostEntity>,
    >,
    ghost_arrows: Query<Entity, With<GhostArrow>>,
    shield_visuals: Query<Entity, With<SpellShieldVisual>>,
) {
    // Filter for game snapshots only (type prefix 0x00), re-queue others
    let raw_data: Vec<Vec<u8>> = connection.incoming_unreliable.drain(..).collect();
    let mut other_data = Vec::new();
    let mut latest_game_data: Option<&[u8]> = None;

    for data in &raw_data {
        if data.is_empty() {
            continue;
        }
        match data[0] {
            UNRELIABLE_GAME_SNAPSHOT => {
                latest_game_data = Some(&data[1..]);
            }
            _ => {
                other_data.push(data.clone());
            }
        }
    }

    // Re-queue non-game data for other systems (spell snapshots)
    connection.incoming_unreliable = other_data;

    let Some(game_bytes) = latest_game_data else {
        return;
    };

    let Ok(snapshot) = bincode::deserialize::<GameSnapshot>(game_bytes) else {
        warn!(
            "Failed to deserialize game snapshot ({} bytes)",
            game_bytes.len()
        );
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

        let material_handle = pick_material(
            &infantry_assets,
            &archer_assets,
            &king_assets,
            &mut materials,
            team,
            is_corpse,
            is_king,
            is_archer,
            is_guard,
        );

        // Sprite-based units keep sprite_mesh for both live and corpse states
        let mesh_handle = if is_king {
            king_assets.sprite_mesh.clone()
        } else if is_archer {
            archer_assets.sprite_mesh.clone()
        } else {
            infantry_assets.sprite_mesh.clone()
        };

        let pos = Vec3::new(unit.x, unit.y, unit.z);

        // Build remote CRDT state from the snapshot
        let remote_crdt = CrdtHealth {
            max_hp: unit.max_hp,
            damage: unit.damage,
            healing: unit.healing,
        };

        let remote_fire = unit.flags & UnitFlags::FIRE_EFFECT != 0;
        let remote_frost = unit.flags & UnitFlags::FROST_EFFECT != 0;
        let remote_electric = unit.flags & UnitFlags::ELECTRIC_EFFECT != 0;
        let remote_spell_shield = unit.flags & UnitFlags::SPELL_SHIELD != 0;

        if let Some(&local_entity) = entity_map.remote_to_local.get(&unit.id) {
            if let Ok((
                entity,
                mut transform,
                material_ref,
                mut crdt_health,
                mut health,
                original_mat,
                has_remote_fire,
                has_remote_frost,
                has_remote_electric,
                has_spell_shield,
                has_corpse,
            )) = ghost_query.get_mut(local_entity)
            {
                transform.translation = pos;

                // Merge CRDT state from host (takes max of each slot)
                crdt_health.merge(&remote_crdt);

                // Re-derive Health from converged CRDT state so damage systems see correct HP
                health.current = crdt_health.current_hp();

                // If a visual effect (fire/frost/electric tint) is active, don't
                // overwrite the tinted material — but update the stored original so
                // the correct base material is restored when the effect expires
                // (e.g., unit becomes a corpse while burning).
                if let Some(orig) = original_mat {
                    if orig.0 != material_handle {
                        commands
                            .entity(entity)
                            .insert(OriginalMaterial(material_handle));
                    }
                } else if material_ref.0 != material_handle {
                    commands
                        .entity(entity)
                        .insert(MeshMaterial3d(material_handle));
                }

                // Sync remote status effect visual markers from host
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

                // Sync spell shield from host
                if remote_spell_shield && !has_spell_shield {
                    commands.entity(entity).insert(SpellShield);
                    // Spawn translucent cross-plane sphere visual as child
                    use crate::game::units::king::constants::{
                        SPELL_SHIELD_COLOR, SPELL_SHIELD_RADIUS,
                    };
                    let shield_visual = commands
                        .spawn((
                            Mesh3d(spell_assets.cross_plane_sphere.clone()),
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color: SPELL_SHIELD_COLOR,
                                unlit: true,
                                alpha_mode: AlphaMode::Blend,
                                ..default()
                            })),
                            Transform::from_scale(Vec3::splat(SPELL_SHIELD_RADIUS)),
                            SpellShieldVisual,
                            OnMultiplayerGameScreen,
                        ))
                        .id();
                    commands.entity(entity).add_child(shield_visual);
                } else if !remote_spell_shield && has_spell_shield {
                    commands.entity(entity).remove::<SpellShield>();
                    // Despawn all shield visuals
                    for vis_entity in &shield_visuals {
                        if let Ok(mut ec) = commands.get_entity(vis_entity) {
                            ec.try_despawn();
                        }
                    }
                }

                // Sync corpse state so spell targeting filters work correctly
                if is_corpse && !has_corpse {
                    commands.entity(entity).insert(Corpse);
                } else if !is_corpse && has_corpse {
                    commands.entity(entity).remove::<Corpse>();
                }
            }
        } else {
            // Spawn new ghost entity with Team, Health, and CrdtHealth for spell targeting
            let initial_health = Health::new(remote_crdt.max_hp);
            let entity = commands
                .spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material_handle),
                    Transform::from_translation(pos),
                    Billboard,
                    GhostEntity,
                    team,
                    NetworkEntityId(unit.id),
                    initial_health,
                    remote_crdt,
                    OnMultiplayerGameScreen,
                ))
                .id();

            // Attach spell shield to newly spawned ghost King if host reports it
            if remote_spell_shield {
                use crate::game::units::king::constants::{
                    SPELL_SHIELD_COLOR, SPELL_SHIELD_RADIUS,
                };
                commands.entity(entity).insert(SpellShield);
                let shield_visual = commands
                    .spawn((
                        Mesh3d(spell_assets.cross_plane_sphere.clone()),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: SPELL_SHIELD_COLOR,
                            unlit: true,
                            alpha_mode: AlphaMode::Blend,
                            ..default()
                        })),
                        Transform::from_scale(Vec3::splat(SPELL_SHIELD_RADIUS)),
                        SpellShieldVisual,
                        OnMultiplayerGameScreen,
                    ))
                    .id();
                commands.entity(entity).add_child(shield_visual);
            }

            if is_corpse {
                commands.entity(entity).insert(Corpse);
            }

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
        if let Some(entity) = entity_map.remove_by_remote(stale_id)
            && let Ok(mut entity_commands) = commands.get_entity(entity)
        {
            entity_commands.try_despawn();
        }
    }

    // Replace all ghost arrows with fresh positions from the snapshot.
    for entity in &ghost_arrows {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.try_despawn();
        }
    }

    for arrow in &snapshot.arrows {
        commands.spawn((
            Mesh3d(archer_assets.arrow_mesh.clone()),
            MeshMaterial3d(archer_assets.arrow_material.clone()),
            Transform::from_translation(Vec3::new(arrow.x, arrow.y, arrow.z)),
            Billboard,
            GhostArrow,
            OnMultiplayerGameScreen,
        ));
    }
}

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
        let mut effects = 0u8;
        if has_fire {
            effects |= UnitFlags::FIRE_EFFECT;
        }
        if has_frost {
            effects |= UnitFlags::FROST_EFFECT;
        }
        if has_electric {
            effects |= UnitFlags::ELECTRIC_EFFECT;
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
