//! Guest-only multiplayer systems.
//!
//! The guest receives unit state snapshots from the host and renders ghost
//! entities that mirror the host's game state. Ghost entities are lightweight
//! visual representations with no simulation components.
//!
//! Spell visual rendering has been moved to `spell_sync.rs` for bidirectional sync.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::game::components::Billboard;
use crate::game::resources::GameOutcome;
use crate::game::units::archer::ArcherAssets;
use crate::game::units::components::OriginalMaterial;
use crate::game::units::components::{
    Corpse, ElectricCharge, FireDoT, FrostEffectMarker, Health, RemoteElectricEffect,
    RemoteFireEffect, RemoteFrostEffect,
};
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::king::components::{SpellShield, SpellShieldVisual};
use crate::game::units::king::resources::KingAssets;
use crate::game::units::wizard::spells::black_hole::components::BlackHole;
use crate::game::units::wizard::spells::entangle::components::EntangleGroundEffect;
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::fog_cloud::components::FogCloudZone;
use crate::game::units::wizard::spells::grease::components::GreaseZone;
use crate::game::units::wizard::spells::healing_plume::components::HealingPlumeZone;
use crate::game::units::wizard::spells::meteor_fall::components::{
    MeteorExplosion, MeteorGroundFire,
};
use crate::game::units::wizard::spells::plague_wind::components::PlagueWindCloud;
use crate::game::units::wizard::spells::spike_growth::components::{
    SpikeGrowthTalentParams, SpikeGrowthZone,
};
use crate::game::units::wizard::spells::squall::components::IceExplosion;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::networking::crdt::CrdtHealth;
use crate::networking::entity_map::{NetworkEntityId, NetworkEntityMap};
use crate::networking::protocol::{GameOverResult, NetworkMessage};
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{
    CrdtSnapshot, CrdtUnitUpdate, GameSnapshot, SpellEffectKind, SpellEffectSnapshot,
    UNRELIABLE_CRDT_SNAPSHOT, UNRELIABLE_GAME_SNAPSHOT, UnitFlags, u8_to_team,
};
use crate::state::MultiplayerGameState;

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
            Has<FrostEffectMarker>,
            Has<ElectricCharge>,
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

/// Picks the correct material handle for a ghost entity.
///
/// Corpses use shared preloaded materials (simple circles, no texture).
/// Live units get per-entity sprite materials with team tinting.
#[allow(clippy::too_many_arguments)]
fn pick_material(
    infantry_assets: &InfantryAssets,
    archer_assets: &ArcherAssets,
    king_assets: &KingAssets,
    materials: &mut Assets<StandardMaterial>,
    team: crate::game::units::components::Team,
    is_corpse: bool,
    is_king: bool,
    is_archer: bool,
    is_guard: bool,
) -> Handle<StandardMaterial> {
    use crate::game::units::components::CORPSE_MATERIAL_VARIANTS;
    use crate::game::units::systems::{corpse_material_for_team, create_default_sprite_material};

    if is_corpse {
        let idx = rand::random::<usize>() % CORPSE_MATERIAL_VARIANTS;
        if is_king {
            king_assets.corpse_materials[idx].clone()
        } else if is_archer {
            corpse_material_for_team(
                &archer_assets.defender_corpse_materials,
                &archer_assets.attacker_corpse_materials,
                &archer_assets.undead_corpse_materials,
                team,
                idx,
            )
        } else {
            corpse_material_for_team(
                &infantry_assets.defender_corpse_materials,
                &infantry_assets.attacker_corpse_materials,
                &infantry_assets.undead_corpse_materials,
                team,
                idx,
            )
        }
    } else if is_king {
        use crate::game::units::king::constants::KING_SPRITE_TINT;
        create_default_sprite_material(
            materials,
            king_assets.sprite_texture.clone(),
            KING_SPRITE_TINT,
        )
    } else if is_archer {
        let tint = crate::game::units::systems::archer_sprite_tint_for_team(team);
        create_default_sprite_material(materials, archer_assets.sprite_texture.clone(), tint)
    } else if is_guard {
        use crate::game::units::infantry::constants::KINGS_GUARD_SPRITE_TINT;
        create_default_sprite_material(
            materials,
            infantry_assets.sprite_texture.clone(),
            KINGS_GUARD_SPRITE_TINT,
        )
    } else {
        let tint = crate::game::units::systems::sprite_tint_for_team(team);
        create_default_sprite_material(materials, infantry_assets.sprite_texture.clone(), tint)
    }
}

/// Spawns a persistent spell effect entity with real components.
///
/// Returns the spawned entity, or `None` if the kind is unknown.
/// Zone fade systems modify materials directly, so zones get unique material clones.
/// Some effects (black hole, lightning rod) allocate meshes at spawn since they need
/// specific sizes; these are rare entities so the cost is negligible.
pub(super) fn spawn_spell_effect(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
) -> Option<Entity> {
    let kind = SpellEffectKind::try_from(effect.kind).ok()?;
    let pos = Vec3::new(effect.x, effect.y, effect.z);
    let extra = effect.extra;

    // Rotation for flat circles (face up on the ground plane)
    let flat_rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);

    match kind {
        // ── Zones (flat circles, real components, unique materials for fading) ──
        SpellEffectKind::SpikeGrowthZone => {
            let radius = extra[0];
            let duration = extra[1];
            Some(
                commands
                    .spawn((
                        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z)),
                        SpikeGrowthZone::new(
                            Vec3::new(pos.x, 0.0, pos.z),
                            radius,
                            0.0,
                            1.0,
                            0.0,
                            0.0,
                            duration,
                            SpikeGrowthTalentParams::default(),
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::HealingPlumeZone => {
            let radius = extra[0];
            let duration = extra[1];
            let material = materials.add(materials.get(&assets.healing_plume_zone)?.clone());
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_circle.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                            .with_rotation(flat_rotation)
                            .with_scale(Vec3::splat(radius)),
                        HealingPlumeZone::new(
                            Vec3::new(pos.x, 0.0, pos.z),
                            radius,
                            0.0,
                            1.0,
                            duration,
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::EntangleGround => {
            let duration = extra[1];
            Some(
                commands
                    .spawn((
                        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z)),
                        Visibility::Hidden,
                        EntangleGroundEffect::new(
                            duration,
                            Vec3::new(pos.x, 1.0, pos.z),
                            120.0,
                            crate::game::units::wizard::spells::entangle::components::EntangleTalentParams::default(),
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::FogCloudZone => {
            let radius = extra[0];
            let duration = extra[1];
            let material = materials.add(materials.get(&assets.fog_cloud_zone)?.clone());
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_circle.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                            .with_rotation(flat_rotation)
                            .with_scale(Vec3::splat(radius)),
                        FogCloudZone::new(
                            Vec3::new(pos.x, 0.0, pos.z),
                            radius,
                            0.0,
                            0.0,
                            1.0,
                            duration,
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::GreaseZone => {
            let radius = extra[0];
            let duration = extra[1];
            let mut base_mat = materials.get(&assets.grease_zone)?.clone();
            base_mat.alpha_mode = bevy::render::alpha::AlphaMode::Mask(0.01);
            let material = materials.add(base_mat);
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_circle.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(Vec3::new(pos.x, 2.0, pos.z))
                            .with_rotation(flat_rotation)
                            .with_scale(Vec3::splat(radius)),
                        GreaseZone::new(
                            Vec3::new(pos.x, 0.0, pos.z),
                            radius,
                            0.0,
                            0.0,
                            1.0,
                            duration,
                            0.0,
                            0.0,
                            0.0,
                            1.0,
                            Default::default(),
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::GreaseFire => {
            // Fire overlay is a second circle on top of the grease zone.
            // Scale is updated every frame from the snapshot (fire spread animation).
            let scale = extra[0].max(0.01);
            let material = materials.add(materials.get(&assets.grease_fire)?.clone());
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_circle.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(Vec3::new(pos.x, 1.1, pos.z))
                            .with_rotation(flat_rotation)
                            .with_scale(Vec3::splat(scale)),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::PlagueWindCloud => {
            let radius = extra[0];
            let duration = extra[1];
            let speed = extra[2];
            let direction_angle = extra[3];
            let direction = Vec3::new(direction_angle.sin(), 0.0, direction_angle.cos());
            let material = materials.add(materials.get(&assets.plague_wind_zone)?.clone());
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_circle.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                            .with_rotation(flat_rotation)
                            .with_scale(Vec3::splat(radius)),
                        PlagueWindCloud::new(
                            Vec3::new(pos.x, 0.0, pos.z),
                            radius,
                            0.0,
                            1.0,
                            duration,
                            speed,
                            direction,
                            Default::default(),
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::MeteorGroundFire => {
            let radius = extra[0];
            let duration = extra[1];
            let material = materials.add(materials.get(&assets.meteor_ground_fire)?.clone());
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_circle.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(Vec3::new(pos.x, 0.5, pos.z))
                            .with_rotation(flat_rotation)
                            .with_scale(Vec3::splat(radius)),
                        MeteorGroundFire::new(
                            Vec3::new(pos.x, 0.0, pos.z),
                            radius,
                            0.0,
                            1.0,
                            duration,
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        // ── Objects (3D meshes, shared materials) ──
        SpellEffectKind::BlackHole => {
            let max_radius = extra[0];
            let empowerment = extra[1];
            // Cross-plane sphere scaled by max_radius * growth_factor in update_black_hole_visuals
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.cross_plane_sphere.clone()),
                        MeshMaterial3d(assets.black_hole.clone()),
                        Transform::from_translation(pos).with_scale(Vec3::ZERO), // Grows from 0 via update_black_hole_visuals
                        BlackHole::new(pos, max_radius, empowerment, Default::default()),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::ArcaneCrystal => {
            let range = extra[0];
            let duration = extra[1];
            let empowerment = extra[2];
            let height = 35.0 * empowerment; // CRYSTAL_HEIGHT * empowerment
            let sphere_radius = height / 3.0;
            // Cross-plane sphere scaled to crystal shape
            Some(commands.spawn((
                Mesh3d(assets.cross_plane_sphere.clone()),
                MeshMaterial3d(assets.arcane_crystal.clone()),
                Transform::from_translation(Vec3::new(pos.x, height / 2.0, pos.z))
                    .with_scale(Vec3::new(0.7 * sphere_radius, 1.5 * sphere_radius, 0.7 * sphere_radius)),
                crate::game::units::wizard::spells::arcane_crystal::components::ArcaneCrystal::new(
                    Vec3::new(pos.x, height / 2.0, pos.z),
                    range, duration, range * 0.15, empowerment,
                ),
                OnMultiplayerGameScreen,
            )).id())
        }

        SpellEffectKind::LightningRod => {
            let duration = extra[0];
            let empowerment = extra[1];
            // Lightning rod uses a cylinder mesh; create one at spawn.
            // This is a small allocation but rods are rare (1-2 at most).
            let tower_height = 60.0; // TOWER_HEIGHT
            let tower_radius = 8.0; // TOWER_RADIUS
            Some(commands.spawn((
                Mesh3d(assets.unit_cuboid.clone()),
                MeshMaterial3d(assets.lightning_rod.clone()),
                Transform::from_translation(Vec3::new(pos.x, tower_height / 2.0, pos.z))
                    .with_scale(Vec3::new(tower_radius * 2.0, tower_height, tower_radius * 2.0)),
                crate::game::units::wizard::spells::lightning_rod::components::LightningRod::new(
                    Vec3::new(pos.x, 0.0, pos.z),
                    duration, empowerment,
                    crate::game::units::wizard::spells::lightning_rod::components::LightningRodTalentParams::default(),
                ),
                OnMultiplayerGameScreen,
            )).id())
        }

        // ── Walls ──
        SpellEffectKind::WallOfStone => {
            let half_length = extra[0];
            let half_width = extra[1];
            let height = extra[2];
            let duration = extra[3];
            let rotation = Quat::from_rotation_y(effect.rotation_y);
            // Reconstruct forward/right from rotation
            let forward = rotation * Vec3::X;
            let right = Vec3::new(-forward.z, 0.0, forward.x);
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_cuboid.clone()),
                        MeshMaterial3d(assets.wall_of_stone.clone()),
                        Transform::from_translation(Vec3::new(pos.x, height / 2.0, pos.z))
                            .with_rotation(rotation)
                            .with_scale(Vec3::new(half_length * 2.0, height, half_width * 2.0)),
                        WallOfStone {
                            center: Vec3::new(pos.x, 0.0, pos.z),
                            half_length,
                            half_width,
                            forward,
                            right,
                            height,
                            time_alive: 0.0,
                            duration,
                            sinking: false,
                            empowerment: 1.0,
                            permanent: false,
                        },
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::WallOfFire => {
            let half_width = extra[0];
            let duration = extra[1];
            let wall_length = extra[2];
            let material = materials.add(StandardMaterial {
                base_color: Color::NONE,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });
            let rotation = Quat::from_rotation_y(effect.rotation_y);
            let wall_height = 10.0;
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_cuboid.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(Vec3::new(pos.x, wall_height / 2.0, pos.z))
                            .with_rotation(rotation)
                            .with_scale(Vec3::new(wall_length, wall_height, 60.0)),
                        WallOfFireEffect::new(
                            Vec3::ZERO,
                            Vec3::ZERO,
                            half_width,
                            0.0,
                            crate::game::units::DamageType::Fire,
                            1.0,
                            duration,
                            Default::default(),
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        // ── Explosions (unit meshes, scale-driven animation) ──
        SpellEffectKind::FireballExplosion => {
            let max_radius = extra[0];
            let empowerment = extra[1];
            // Raise slightly above ground so cross-plane sphere is visible
            let explosion_pos = Vec3::new(pos.x, pos.y.max(5.0), pos.z);
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.cross_plane_sphere.clone()),
                        MeshMaterial3d(assets.fireball_explosion.clone()),
                        Transform::from_translation(explosion_pos).with_scale(Vec3::splat(0.1)),
                        FireballExplosion::new(
                            pos,
                            max_radius,
                            0.0,
                            crate::game::units::DamageType::Fire,
                            empowerment,
                        ),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::MeteorExplosion => {
            let max_radius = extra[0];
            let material = materials.add(materials.get(&assets.meteor_explosion)?.clone());
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_circle.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                            .with_rotation(flat_rotation)
                            .with_scale(Vec3::splat(0.1)),
                        MeteorExplosion::new(pos, max_radius, 0.0),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
        }

        SpellEffectKind::IceExplosion => {
            let max_radius = extra[0];
            let empowerment = extra[1];
            Some(
                commands
                    .spawn((
                        Mesh3d(assets.unit_circle.clone()),
                        MeshMaterial3d(assets.ice_explosion.clone()),
                        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                            .with_rotation(flat_rotation)
                            .with_scale(Vec3::splat(0.1)),
                        IceExplosion::new(pos, max_radius, 0.0, empowerment),
                        OnMultiplayerGameScreen,
                    ))
                    .id(),
            )
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
