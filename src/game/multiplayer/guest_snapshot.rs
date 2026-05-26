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

/// Guest-side forwarder: when a spell on the guest applies damage to a
/// ghost unit, SP's `apply_spell_damage` inserts a `PendingDamageEffect`
/// on the target. We **poll** for any such component (deliberately not
/// `Added<>`) each frame because `apply_spell_damage` queues inserts via
/// `Commands`; the component isn't visible to an `Added` filter until
/// after the next command flush, and a ghost killed before the flush
/// would never be forwarded. Polling catches both same-frame inserts and
/// any that survive across a frame.
///
/// The host then owns status-effect bookkeeping (DoT stacks, durations,
/// snapshot flags). After forwarding, the local `PendingDamageEffect` is
/// removed so the guest doesn't *also* tick the DoT (which would double-
/// apply on top of the host-ticked damage that propagates back via CRDT).
///
/// Excremage conversion: a guest playing Excremage should turn every spell
/// hit into a Poop hit, but `process_pending_damage_effects` does that
/// lookup against the LOCAL `GameConfig.wizard_type` — and on the host
/// that's the host's wizard, not the guest's. So we do the conversion here
/// on the guest before forwarding, using the guest's own config.
pub fn forward_spell_hits_to_host(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    config: Res<crate::config::GameConfig>,
    hits: Query<
        (
            Entity,
            &crate::networking::entity_map::NetworkEntityId,
            &crate::game::units::components::PendingDamageEffect,
        ),
        With<super::components::GhostEntity>,
    >,
) {
    let excremage =
        config.wizard_type == crate::config::WizardType::Excremage;
    for (entity, net_id, pending) in &hits {
        let damage_type = if excremage {
            crate::game::units::damage::DamageType::Poop
        } else {
            pending.damage_type
        };
        connection
            .outgoing_messages
            .push(crate::networking::protocol::NetworkMessage::SpellHitUnit {
                target_network_id: net_id.0,
                damage: pending.damage,
                damage_type: damage_type.to_u8(),
            });
        commands
            .entity(entity)
            .remove::<crate::game::units::components::PendingDamageEffect>();
    }
}

/// Receives the latest unit state snapshot from the host and creates/updates/despawns
/// ghost entities to match the host's game state.
///
/// Filters incoming unreliable data by type prefix byte, processing only game
/// snapshots (unit data). Spell visual snapshots are handled by `spell_sync.rs`.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn apply_state_snapshot(
    mut commands: Commands,
    // Real (wall) time, NOT virtual time: the velocity synthesis below
    // computes `delta / dt` and dividing by virtual-time delta blows up when
    // the player has `config.game_speed = 0.0` or the game is otherwise
    // virtually paused, even though real time keeps advancing and snapshots
    // keep arriving.
    time: Res<Time<bevy::time::Real>>,
    // Wall-clock time of the most recent snapshot we processed. Used to
    // decay ghost velocities toward zero during extended snapshot outages
    // so units don't appear to walk-in-place forever during a network
    // blip. `0.0` sentinel = "no snapshot yet, don't decay anything."
    mut last_snapshot_real_time: Local<f32>,
    mut connection: ResMut<NetworkConnection>,
    mut entity_map: ResMut<NetworkEntityMap>,
    infantry_assets: Res<InfantryAssets>,
    archer_assets: Res<ArcherAssets>,
    king_assets: Res<KingAssets>,
    spell_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut ghost_query: Query<
        (
            Entity,
            &mut Transform,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut CrdtHealth,
            &mut Health,
            &mut crate::game::components::Velocity,
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

    // Ghost velocities deliberately persist between snapshots — when a
    // packet drops, continuing the previous animation looks better than
    // flickering to idle. The host's "unit stopped" tick will eventually
    // send delta=0 → velocity=0 → idle on the correct frame.
    let Some(game_bytes) = latest_game_data else {
        return;
    };
    *last_snapshot_real_time = time.elapsed_secs();

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
                mut velocity,
                original_mat,
                has_remote_fire,
                has_remote_frost,
                has_remote_electric,
                has_spell_shield,
                has_corpse,
            )) = ghost_query.get_mut(local_entity)
            {
                // Synthesise a per-second velocity from the snapshot delta
                // so the shared animation systems (walking, facing) treat
                // the ghost the same as a host-simulated unit.
                //
                // Clamp the XZ magnitude to `MAX_SYNTH_SPEED` so a single
                // large jump (teleport, knockback, lag spike) doesn't
                // produce a 1000+ u/s spike that flashes the walking pose
                // and snaps facing for one frame before the next snapshot
                // zeros it. The cap is set well above any natural unit
                // movement speed so normal motion is never throttled.
                const MAX_SYNTH_SPEED: f32 = 400.0;
                let delta = pos - transform.translation;
                let dt = time.delta_secs().max(1.0e-4);
                let raw_xz = Vec3::new(delta.x, 0.0, delta.z) / dt;
                let speed = raw_xz.length();
                let clamped_xz = if speed > MAX_SYNTH_SPEED {
                    raw_xz * (MAX_SYNTH_SPEED / speed)
                } else {
                    raw_xz
                };
                velocity.x = clamped_xz.x;
                velocity.z = clamped_xz.z;

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

                // Sync corpse state so spell targeting filters work correctly.
                // On the non-corpse → corpse transition, also kick off the
                // shared `DyingAnimation` so the ghost plays the death frames
                // before settling into its corpse pose — matching the SP
                // visual where units don't pop straight from standing to
                // laid-flat. King has no death sprite sheet (instant corpse
                // swap in SP), so it's skipped here.
                if is_corpse && !has_corpse {
                    let mut ec = commands.entity(entity);
                    ec.insert(Corpse);
                    if !is_king {
                        let death_texture = if is_archer {
                            archer_assets.death_texture.clone()
                        } else {
                            infantry_assets.death_texture.clone()
                        };
                        ec.insert(crate::game::units::components::DyingAnimation::new(
                            death_texture,
                        ));
                    }
                } else if !is_corpse && has_corpse {
                    commands.entity(entity).remove::<Corpse>();
                }
            }
        } else {
            // Spawn new ghost entity with Team, Health, and CrdtHealth for spell targeting.
            // Animation components (Velocity + FacingDirection + WalkingAnimation) let the
            // shared animation systems run for ghosts the same way they run for
            // host-simulated units — Velocity is synthesised in this system on subsequent
            // snapshots from the position delta.
            // Give the ghost the SAME `Hitbox` SP uses for this unit type
            // so SP spell-collision systems (fireball, ice, etc.) running
            // on this peer can land on it. Damage applied to the ghost's
            // `Health` flows through `sync_health_to_crdt` → CRDT slot →
            // peer snapshot → host's authoritative units lose HP. This is
            // the entire reason guest-cast spells now reach host units
            // without any per-spell network-message plumbing.
            //
            // **Known gap (latent):** today MP only spawns infantry / archer
            // / king / king's-guard, all covered by this three-way branch.
            // If MP ever spawns brutes / healers / dispellers / etc., add
            // their unit type to `UnitFlags` and a corresponding branch
            // here — otherwise their ghosts get the infantry hitbox (32u
            // vs e.g. brute's 80u) and spells will miss them.
            use crate::game::units::components::Hitbox;
            let hitbox = if is_king {
                Hitbox::new(
                    crate::game::units::king::constants::KING_RADIUS,
                    crate::game::units::king::constants::KING_HITBOX_HEIGHT,
                )
            } else if is_archer {
                Hitbox::new(
                    crate::game::units::archer::constants::ARCHER_RADIUS,
                    crate::game::constants::DEFENDER_HITBOX_HEIGHT,
                )
            } else {
                // Infantry and king's guards both use the standard unit radius.
                Hitbox::new(
                    crate::game::units::infantry::constants::UNIT_RADIUS,
                    crate::game::constants::DEFENDER_HITBOX_HEIGHT,
                )
            };

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
                    hitbox,
                    OnMultiplayerGameScreen,
                    crate::game::components::Velocity::default(),
                    crate::game::units::components::FacingDirection::default(),
                    // Stagger the walk cycle so a freshly-snapshot army
                    // doesn't flip animation frames in unison.
                    crate::game::units::components::WalkingAnimation::new_staggered(
                        &mut game_rng.0,
                    ),
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
