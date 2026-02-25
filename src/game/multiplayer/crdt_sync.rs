//! CRDT-Health synchronization systems.
//!
//! These systems bridge the gap between the ECS `Health` component (used by all
//! existing damage, healing, and death systems) and `CrdtHealth` (used for
//! multiplayer state convergence). They run on both host and guest.
//!
//! **How it works:**
//! 1. `attach_crdt_health` — Automatically adds `CrdtHealth` to any entity that
//!    has `Health` but not yet `CrdtHealth`. Runs early so new units get CRDT
//!    tracking immediately.
//! 2. `sync_health_to_crdt` — Runs after all damage/healing systems. Detects
//!    local changes to `Health.current` (from melee, spells, DoTs, healing) and
//!    records them in the local peer's CRDT slot. Then re-derives `Health.current`
//!    from the full CRDT state (which includes the remote peer's contributions).

use bevy::prelude::*;

use crate::game::pathfinding::{ObstacleChanged, ObstacleType};
use crate::game::units::components::Health;
use crate::networking::crdt::{CrdtHealth, PeerId};
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::NetworkConnection;

/// Automatically attaches `CrdtHealth` to any entity with `Health` that doesn't
/// have one yet. This avoids modifying every unit spawn function.
pub fn attach_crdt_health(
    mut commands: Commands,
    new_units: Query<(Entity, &Health), Without<CrdtHealth>>,
) {
    for (entity, health) in &new_units {
        commands.entity(entity).insert(CrdtHealth::new(health.max));
    }
}

/// Synchronizes local Health changes into CrdtHealth and re-derives Health from
/// the converged CRDT state.
///
/// This system must run **after** all damage and healing systems so it can detect
/// the delta between the CRDT-expected HP and the actual Health.current value.
///
/// Flow:
/// 1. Compute expected HP from CRDT: `crdt.current_hp()`
/// 2. Compare with actual `health.current` (modified by damage/healing systems)
/// 3. If health decreased → local damage occurred → `crdt.apply_damage(peer, delta)`
/// 4. If health increased → local healing occurred → `crdt.apply_healing(peer, delta)`
/// 5. Re-derive `health.current = crdt.current_hp()` (includes remote peer's data)
pub fn sync_health_to_crdt(peer_id: Res<PeerId>, mut units: Query<(&mut Health, &mut CrdtHealth)>) {
    let peer = peer_id.0;

    for (mut health, mut crdt) in &mut units {
        let crdt_hp = crdt.current_hp();

        if health.current < crdt_hp - f32::EPSILON {
            // Health decreased — local damage occurred
            let local_damage = crdt_hp - health.current;
            crdt.apply_damage(peer, local_damage);
        } else if health.current > crdt_hp + f32::EPSILON {
            // Health increased — local healing occurred
            let local_healing = health.current - crdt_hp;
            crdt.apply_healing(peer, local_healing);
        }

        // Re-derive health from the CRDT (includes remote peer's contributions)
        health.current = crdt.current_hp();
    }
}

/// Processes incoming `WallPlaced` messages and writes local `ObstacleChanged`
/// messages so the pathfinding grid updates for walls placed by the remote peer.
pub fn receive_wall_placement(
    mut connection: ResMut<NetworkConnection>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }

    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::WallPlaced { bounds, placed } => {
                let obstacle_type = if placed {
                    ObstacleType::Blocked
                } else {
                    ObstacleType::Removed
                };
                obstacle_events.write(ObstacleChanged {
                    bounds: Rect::new(bounds[0], bounds[1], bounds[2], bounds[3]),
                    obstacle_type,
                });
            }
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}
