//! Host-only multiplayer systems.
//!
//! These systems run on the host to assign network IDs to entities and
//! send state snapshots to the guest every frame.

use bevy::prelude::*;

use crate::game::resources::GameOutcome;
use crate::game::units::archer::components::Arrow;
use crate::game::units::archer::Archer;
use crate::game::units::components::{Corpse, Health, KingsGuard, Team};
use crate::game::units::king::components::King;
use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::magic_missile::components::MagicMissile;
use crate::networking::entity_map::{EntityIdCounter, NetworkEntityId};
use crate::networking::protocol::{GameOverResult, NetworkMessage};
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::{
    ArrowSnapshot, BeamSnapshot, GameSnapshot, MagicMissileSnapshot, SnapshotTick,
    build_unit_snapshot,
};
use crate::state::MultiplayerGameState;

/// Assigns `NetworkEntityId` to newly spawned entities that have `Health` + `Team`
/// but don't yet have a network ID.
pub fn assign_network_ids(
    mut commands: Commands,
    mut counter: ResMut<EntityIdCounter>,
    new_units: Query<Entity, (With<Health>, With<Team>, Without<NetworkEntityId>)>,
) {
    for entity in &new_units {
        let net_id = counter.next();
        commands.entity(entity).insert(net_id);
    }
}

/// Serializes the full game state and sends it over the unreliable channel.
///
/// Runs every frame (~60Hz). Queries all entities with a `NetworkEntityId` and
/// builds a compact `GameSnapshot` serialized with `bincode`.
#[allow(clippy::type_complexity)]
pub fn send_state_snapshots(
    mut tick: ResMut<SnapshotTick>,
    mut connection: ResMut<NetworkConnection>,
    units: Query<(
        &NetworkEntityId,
        &Transform,
        &Team,
        &Health,
        Has<Corpse>,
        Has<King>,
        Has<Archer>,
        Has<KingsGuard>,
    )>,
    arrows: Query<&Transform, With<Arrow>>,
    missiles: Query<&Transform, With<MagicMissile>>,
    beams: Query<&DisintegrateBeam>,
) {
    tick.0 = tick.0.wrapping_add(1);

    let mut snapshot = GameSnapshot {
        tick: tick.0,
        units: Vec::with_capacity(units.iter().len()),
        arrows: Vec::with_capacity(arrows.iter().len()),
        magic_missiles: Vec::with_capacity(missiles.iter().len()),
        beams: Vec::with_capacity(beams.iter().len()),
    };

    for (net_id, transform, team, health, is_corpse, is_king, is_archer, is_guard) in &units {
        snapshot.units.push(build_unit_snapshot(
            net_id, transform, team, health, is_corpse, is_king, is_archer, is_guard,
        ));
    }

    for transform in &arrows {
        snapshot.arrows.push(ArrowSnapshot {
            x: transform.translation.x,
            y: transform.translation.y,
            z: transform.translation.z,
        });
    }

    for transform in &missiles {
        snapshot.magic_missiles.push(MagicMissileSnapshot {
            x: transform.translation.x,
            y: transform.translation.y,
            z: transform.translation.z,
        });
    }

    for beam in &beams {
        snapshot.beams.push(BeamSnapshot {
            ox: beam.origin.x,
            oy: beam.origin.y,
            oz: beam.origin.z,
            dx: beam.direction.x,
            dy: beam.direction.y,
            dz: beam.direction.z,
            length: beam.current_length(),
        });
    }

    if let Ok(data) = bincode::serialize(&snapshot) {
        connection.outgoing_unreliable.push(data);
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
