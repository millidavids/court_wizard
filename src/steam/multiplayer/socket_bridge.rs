//! The Steam-side data pump: ferries bytes between the `NetConnection` and
//! `NetworkConnection`'s reliable + unreliable queues.
//!
//! Mirrors the iroh `transport/bridge.rs` for `ConnectionMode::Steam`. Socket
//! lifecycle (listen / dial / tear down) lives in `sockets.rs`.

use bevy::prelude::*;
use bevy_steamworks::networking_types::SendFlags;

use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{ConnectionMode, ConnectionState, NetworkConnection};

use super::constants::{RECEIVE_BATCH, TAG_RELIABLE, TAG_UNRELIABLE};
use super::sockets::SteamP2pSocket;

/// Drain outgoing reliable + unreliable queues from `NetworkConnection` into
/// the Steam `NetConnection`, and route incoming `NetworkingMessage`s back
/// into the incoming queues. Mirrors the iroh `transport_bridge_system` but
/// uses a 1-byte channel tag (0x01 = reliable, 0x02 = unreliable) since
/// `NetworkingMessage::send_flags()` doesn't reliably distinguish unreliable
/// variants on the receive side (see `networking_types.rs:1879`).
pub(super) fn steam_transport_bridge_system(
    mut socket: Option<ResMut<SteamP2pSocket>>,
    mut connection: ResMut<NetworkConnection>,
) {
    if connection.mode != ConnectionMode::Steam {
        return;
    }
    let Some(socket) = socket.as_deref_mut() else {
        return;
    };
    if socket.connection.is_none() {
        return;
    }

    // Drain the outgoing queues only when they actually hold something. The
    // `is_empty()` checks go through `Deref` (immutable), so an idle frame never
    // touches `NetworkConnection` mutably — without this guard, `drain(..)` would
    // mark the resource changed EVERY frame, and the multiplayer tab's
    // `rebuild_multiplayer_on_lobby_change` (gated on `resource_changed::<NetworkConnection>`)
    // would despawn + respawn its panels every frame, destroying button entities
    // before a click (press one frame, release the next) could ever complete. The
    // iroh bridge guards the same way (see `transport/bridge.rs`).

    // --- Send outgoing reliable -------------------------------------------
    if !connection.outgoing_messages.is_empty() {
        let outgoing_msgs: Vec<NetworkMessage> = std::mem::take(&mut connection.outgoing_messages);
        for msg in outgoing_msgs {
            match bincode::serialize(&msg) {
                Ok(payload) => {
                    let mut framed = Vec::with_capacity(payload.len() + 1);
                    framed.push(TAG_RELIABLE);
                    framed.extend_from_slice(&payload);
                    if let Some(net_conn) = socket.connection.as_ref()
                        && let Err(err) = net_conn.send_message(&framed, SendFlags::RELIABLE)
                    {
                        warn!("[Steam MP] send_message (reliable) failed: {err:?}");
                    }
                }
                Err(err) => {
                    warn!("[Steam MP] Failed to serialize outgoing message: {err}");
                }
            }
        }
    }

    // --- Send outgoing unreliable -----------------------------------------
    if !connection.outgoing_unreliable.is_empty() {
        let outgoing_unrel: Vec<Vec<u8>> = std::mem::take(&mut connection.outgoing_unreliable);
        for payload in outgoing_unrel {
            let mut framed = Vec::with_capacity(payload.len() + 1);
            framed.push(TAG_UNRELIABLE);
            framed.extend_from_slice(&payload);
            if let Some(net_conn) = socket.connection.as_ref()
                && let Err(err) =
                    net_conn.send_message(&framed, SendFlags::UNRELIABLE | SendFlags::NO_NAGLE)
            {
                warn!("[Steam MP] send_message (unreliable) failed: {err:?}");
            }
        }
    }

    // --- Drain incoming ---------------------------------------------------
    // `receive_messages` takes `&mut NetConnection`; reborrow mutably.
    let receive_result = socket
        .connection
        .as_mut()
        .map(|c| c.receive_messages(RECEIVE_BATCH));

    match receive_result {
        Some(Ok(messages)) => {
            for msg in messages {
                let bytes = msg.data();
                // Need at least the tag byte plus one payload byte.
                if bytes.len() < 2 {
                    warn!(
                        "[Steam MP] Dropping {}-byte message (need >= 2: tag + payload)",
                        bytes.len()
                    );
                    continue;
                }
                let tag = bytes[0];
                let payload = &bytes[1..];
                match tag {
                    t if t == TAG_RELIABLE => match bincode::deserialize::<NetworkMessage>(payload)
                    {
                        Ok(decoded) => connection.incoming_messages.push(decoded),
                        Err(err) => {
                            warn!("[Steam MP] Failed to deserialize incoming message: {err}");
                        }
                    },
                    t if t == TAG_UNRELIABLE => {
                        connection.incoming_unreliable.push(payload.to_vec());
                    }
                    other => {
                        warn!("[Steam MP] Unknown channel tag {other:#x} — dropping message");
                    }
                }
            }
        }
        Some(Err(_)) => {
            warn!("[Steam MP] receive_messages returned InvalidHandle — flagging disconnect");
            connection.state = ConnectionState::Disconnected;
        }
        None => {}
    }
}
