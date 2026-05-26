//! Host-only multiplayer systems.
//!
//! These systems run on the host to assign network IDs to entities and
//! send unit state snapshots to the guest every frame.

use bevy::prelude::*;

use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::resources::GameOutcome;
use crate::game::units::archer::Archer;
use crate::game::units::archer::components::Arrow;
use crate::game::units::components::{
    Corpse, FireDoT, FrostAccumulation, Health, KingsGuard, RemoteElectricEffect, RemoteFireEffect,
    RemoteFrostEffect, Shocked, Team,
};
use crate::game::units::king::components::{King, SpellShield};
use crate::networking::crdt::CrdtHealth;
use crate::networking::entity_map::{EntityIdCounter, NetworkEntityId};
use crate::networking::protocol::{GameOverResult, NetworkMessage};
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{
    ArrowSnapshot, CrdtSnapshot, GameSnapshot, SnapshotTick, UNRELIABLE_CRDT_SNAPSHOT, UnitFlags,
    build_unit_snapshot,
};
use crate::state::MultiplayerGameState;

/// Assigns `NetworkEntityId` to newly spawned entities that have `Health` + `Team`
/// or `NetworkedSpellEffect` but don't yet have a network ID.
pub fn assign_network_ids(
    mut commands: Commands,
    mut counter: ResMut<EntityIdCounter>,
    new_units: Query<Entity, (With<Health>, With<Team>, Without<NetworkEntityId>)>,
    new_effects: Query<Entity, (With<NetworkedSpellEffect>, Without<NetworkEntityId>)>,
) {
    for entity in new_units.iter().chain(new_effects.iter()) {
        let net_id = counter.next();
        commands.entity(entity).insert(net_id);
    }
}

/// Serializes unit state and sends it over the unreliable channel.
///
/// Runs every frame (~60Hz). Queries all entities with a `NetworkEntityId` and
/// builds a compact `GameSnapshot` serialized with `bincode`, prefixed with
/// a type byte so the guest can distinguish it from spell visual snapshots.
#[allow(clippy::type_complexity)]
pub fn send_state_snapshots(
    mut tick: ResMut<SnapshotTick>,
    mut connection: ResMut<NetworkConnection>,
    units: Query<(
        &NetworkEntityId,
        &Transform,
        // Host-authoritative velocity — ships in the snapshot so the guest
        // can write it directly onto ghost units instead of synthesising
        // from position deltas (which jitters whenever the host's unit
        // briefly stops, causing the walking animation to reset to idle).
        &crate::game::components::Velocity,
        &Team,
        &Health,
        Option<&CrdtHealth>,
        Has<Corpse>,
        Has<King>,
        Has<Archer>,
        Has<KingsGuard>,
        Has<FireDoT>,
        Has<FrostAccumulation>,
        Has<Shocked>,
        Has<SpellShield>,
    )>,
    arrows: Query<&Transform, With<Arrow>>,
) {
    tick.0 = tick.0.wrapping_add(1);

    let mut snapshot = GameSnapshot {
        tick: tick.0,
        units: Vec::with_capacity(units.iter().len()),
        arrows: Vec::with_capacity(arrows.iter().len()),
    };

    for (
        net_id,
        transform,
        velocity,
        team,
        health,
        crdt_health,
        is_corpse,
        is_king,
        is_archer,
        is_guard,
        has_fire,
        has_frost,
        has_electric,
        has_spell_shield,
    ) in &units
    {
        snapshot.units.push(build_unit_snapshot(
            net_id,
            transform,
            velocity,
            team,
            health,
            crdt_health,
            is_corpse,
            is_king,
            is_archer,
            is_guard,
            has_fire,
            has_frost,
            has_electric,
            has_spell_shield,
        ));
    }

    for transform in &arrows {
        snapshot.arrows.push(ArrowSnapshot {
            x: transform.translation.x,
            y: transform.translation.y,
            z: transform.translation.z,
        });
    }

    if let Ok(data) = bincode::serialize(&snapshot) {
        let mut prefixed = Vec::with_capacity(1 + data.len());
        prefixed.push(crate::networking::snapshot::UNRELIABLE_GAME_SNAPSHOT);
        prefixed.extend_from_slice(&data);
        connection.outgoing_unreliable.push(prefixed);
    }
}

/// Checks for King death and triggers game-over transition.
///
/// Host = Defenders, Guest = Attackers. When a King becomes a corpse:
/// - Defender King dies → guest wins
/// - Attacker King dies → host wins
pub fn check_mp_king_death(
    mut connection: ResMut<NetworkConnection>,
    mut game_outcome: ResMut<GameOutcome>,
    mut next_state: ResMut<NextState<MultiplayerGameState>>,
    dead_kings: Query<&Team, (With<King>, With<Corpse>)>,
) {
    if let Some(team) = dead_kings.iter().next() {
        let result = match team {
            Team::Defenders => GameOverResult::GuestWins,
            Team::Attackers | Team::Undead => GameOverResult::HostWins,
        };

        *game_outcome = match result {
            GameOverResult::HostWins => GameOutcome::Victory,
            GameOverResult::GuestWins => GameOutcome::DefeatKingDied,
        };

        connection
            .outgoing_messages
            .push(NetworkMessage::GameOver(result));
        next_state.set(MultiplayerGameState::ScoreScreen);
    }
}

/// Receives `TeleportUnits` messages from the guest and executes the teleport on the host.
///
/// Unit positions are host-authoritative, so when the guest casts Teleport it sends
/// a message with source/dest/radius. The host applies the actual position changes.
pub fn receive_teleport_message(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    units_query: Query<
        (
            Entity,
            &Transform,
            Option<&crate::game::units::components::Team>,
        ),
        (
            With<crate::game::units::components::Teleportable>,
            Without<
                crate::game::units::wizard::spells::teleport::components::TeleportDestinationCircle,
            >,
            Without<crate::game::units::wizard::spells::teleport::components::TeleportSourceCircle>,
            Without<crate::game::units::components::Corpse>,
        ),
    >,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }

    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::TeleportUnits {
                source_x,
                source_z,
                dest_x,
                dest_z,
                radius,
            } => {
                let source = Vec3::new(source_x, 0.0, source_z);
                let dest = Vec3::new(dest_x, 0.0, dest_z);
                crate::game::units::wizard::spells::teleport::systems::teleport_units_with_radius(
                    &mut rand::rng(),
                    source,
                    dest,
                    radius,
                    &units_query,
                    &mut commands,
                );
            }
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}

/// Receives `SpellHitUnit` messages from the guest and inserts the standard
/// `PendingDamageEffect` on the matching authoritative unit. From there the
/// host runs SP's full status-effect pipeline (`process_pending_damage_effects`
/// → `FireDoT` / `Shocked` / etc. → `update_fire_dot` → CRDT damage tick),
/// and the resulting status flag is shipped back to the guest in the next
/// state snapshot for visual rendering — the guest never owns status state
/// itself.
pub fn receive_spell_hit_messages(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    // Restrict to entities that actually have `Health` — this excludes
    // `NetworkedSpellEffect` entities (walls / zones) that share the
    // `NetworkEntityId` counter but have no use for a status effect.
    units: Query<(Entity, &NetworkEntityId), With<Health>>,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }

    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::SpellHitUnit {
                target_network_id,
                damage,
                damage_type,
            } => {
                // The host owns `NetworkEntityId` for every unit (assigned by
                // `assign_network_ids`); scan for the matching one. ~200
                // units max, trivial.
                let Some(local_entity) = units
                    .iter()
                    .find_map(|(e, id)| (id.0 == target_network_id).then_some(e))
                else {
                    // Unit may have despawned between the guest's hit
                    // detection and this message arriving — silently drop.
                    continue;
                };
                if let Ok(mut ec) = commands.get_entity(local_entity) {
                    ec.insert(crate::game::units::components::PendingDamageEffect {
                        damage,
                        damage_type:
                            crate::game::units::damage::DamageType::from_u8(damage_type),
                    });
                }
            }
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}

/// Receives CRDT health updates from the guest and merges into local unit state.
///
/// The guest sends its CRDT counters (guest spell damage/healing) over the
/// unreliable channel. The host merges these using element-wise max so that
/// guest spell effects are reflected in the host's simulation.
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
    // Filter for CRDT snapshots, re-queue everything else
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

    // Build a temporary lookup: network_id → index in snapshot
    let mut update_map = std::collections::HashMap::with_capacity(snapshot.units.len());
    for (i, update) in snapshot.units.iter().enumerate() {
        update_map.insert(update.id, i);
    }

    // Merge guest CRDT state into host entities and update Health to match.
    // We must update Health.current here so that sync_health_to_crdt doesn't
    // interpret the guest's damage as "local healing" (health.current > crdt_hp).
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

            // Sync remote status effect visual markers from guest
            let remote_fire = update.effects & UnitFlags::FIRE_EFFECT != 0;
            let remote_frost = update.effects & UnitFlags::FROST_EFFECT != 0;
            let remote_electric = update.effects & UnitFlags::ELECTRIC_EFFECT != 0;

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
