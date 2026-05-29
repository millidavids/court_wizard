//! SteamNetworkingSockets P2P bridge: listen socket (host), `connect_p2p`
//! (guest), and the send/recv pump that ferries bytes between the
//! `NetConnection` and `NetworkConnection`'s reliable + unreliable queues.

use bevy::prelude::*;
use bevy_steamworks::networking_sockets::{ListenSocket, NetConnection};
use bevy_steamworks::networking_types::{
    AppNetConnectionEnd, ListenSocketEvent, NetConnectionEnd, NetworkingConnectionState,
    NetworkingIdentity, SendFlags,
};
use bevy_steamworks::{Client, SteamId};
use std::sync::Mutex;

use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{ConnectionMode, ConnectionState, NetworkConnection, PeerRole};

use super::constants::{RECEIVE_BATCH, TAG_RELIABLE, TAG_UNRELIABLE, VIRTUAL_PORT};

/// The Steam-side P2P socket state for the active session.
///
/// `listener` is `!Sync` (its internal `std::sync::mpsc::Receiver` is `!Sync`),
/// so it's wrapped in a `Mutex` to satisfy Bevy's `Resource: Send + Sync`
/// bound. `NetConnection` already provides `unsafe impl Send + Sync` upstream.
#[derive(Resource, Default)]
pub(crate) struct SteamP2pSocket {
    pub(super) listener: Mutex<Option<ListenSocket>>,
    pub(super) connection: Option<NetConnection>,
    pub(super) expected_peer: Option<SteamId>,
}

/// Drop the socket + listener and clear the expected-peer marker. Order
/// matters: the `NetConnection` is dropped before the `ListenSocket` so the
/// listener's tracked-connections set stays in a sane state.
pub(crate) fn tear_down_socket(socket: &mut SteamP2pSocket) {
    socket.connection.take();
    if let Ok(mut listener) = socket.listener.lock() {
        listener.take();
    }
    socket.expected_peer = None;
}

/// Host-side: open a listen socket on `VIRTUAL_PORT` and record the friend
/// we expect to see as the incoming peer. On failure, surface the error via
/// `NetworkConnection` so `sync_lobby_with_connection` flips the UI to Failed.
pub(super) fn start_listening(
    client: &Client,
    socket: &mut SteamP2pSocket,
    connection: &mut NetworkConnection,
    peer_steam_id: SteamId,
) {
    socket.expected_peer = Some(peer_steam_id);
    match client
        .networking_sockets()
        .create_listen_socket_p2p(VIRTUAL_PORT, [])
    {
        Ok(listener) => {
            if let Ok(mut slot) = socket.listener.lock() {
                *slot = Some(listener);
            }
            info!(
                "[Steam MP] Listen socket open on virtual port {VIRTUAL_PORT}, expecting peer {}",
                peer_steam_id.raw()
            );
        }
        Err(_) => {
            warn!("[Steam MP] Failed to create P2P listen socket");
            socket.expected_peer = None;
            connection.error = Some(
                "Steam couldn't open a P2P listen socket. Try again or restart the game."
                    .to_string(),
            );
            connection.state = ConnectionState::Failed;
        }
    }
}

/// Drive the host's listen socket: accept the expected friend's `Connecting`
/// event, store the resulting `NetConnection` on `Connected`, tear down on
/// `Disconnected`.
pub(super) fn drive_steam_listen_socket(
    mut socket: Option<ResMut<SteamP2pSocket>>,
    mut connection: ResMut<NetworkConnection>,
) {
    let Some(socket) = socket.as_deref_mut() else {
        return;
    };

    // Drain all events first. We hold the lock briefly per drain to keep
    // the borrow non-overlapping with mutations of `socket.connection` and
    // `socket.expected_peer` below.
    let events: Vec<ListenSocketEvent> = {
        let Ok(slot) = socket.listener.lock() else {
            return;
        };
        let Some(listener) = slot.as_ref() else {
            return;
        };
        let mut events = Vec::new();
        while let Some(event) = listener.try_receive_event() {
            events.push(event);
        }
        events
    };

    for event in events {
        match event {
            ListenSocketEvent::Connecting(req) => {
                let remote_steam_id = req.remote().steam_id();
                if remote_steam_id != socket.expected_peer {
                    warn!(
                        "[Steam MP] Rejecting stranger connect attempt from {:?} (expected {:?})",
                        remote_steam_id, socket.expected_peer
                    );
                    let _ = req.reject(
                        NetConnectionEnd::App(AppNetConnectionEnd::generic_normal()),
                        Some("not the expected friend"),
                    );
                    continue;
                }
                if let Err(err) = req.accept() {
                    warn!("[Steam MP] Accept failed: {err:?}");
                    tear_down_socket(socket);
                    connection.error =
                        Some("Steam couldn't accept the incoming connection.".to_string());
                    connection.state = ConnectionState::Failed;
                    return;
                }
            }
            ListenSocketEvent::Connected(connected) => {
                info!(
                    "[Steam MP] Listen socket accepted peer {:?}",
                    connected.remote().steam_id()
                );
                socket.connection = Some(connected.take_connection());
                connection.state = ConnectionState::Connected;
            }
            ListenSocketEvent::Disconnected(disc) => {
                info!(
                    "[Steam MP] Peer disconnected (end_reason={:?})",
                    disc.end_reason()
                );
                tear_down_socket(socket);
                connection.state = ConnectionState::Disconnected;
            }
        }
    }
}

/// Guest-side: initiate `connect_p2p` to the host's SteamId. Stores the
/// resulting `NetConnection` on the resource; the polling system below picks
/// up the eventual `Connected` transition. On failure, surface the error via
/// `NetworkConnection` so the UI flips to Failed instead of spinning forever.
pub(super) fn start_connecting(
    client: &Client,
    socket: &mut SteamP2pSocket,
    connection: &mut NetworkConnection,
    host_steam_id: SteamId,
) {
    socket.expected_peer = Some(host_steam_id);
    let identity = NetworkingIdentity::new_steam_id(host_steam_id);
    match client
        .networking_sockets()
        .connect_p2p(identity, VIRTUAL_PORT, [])
    {
        Ok(net_connection) => {
            socket.connection = Some(net_connection);
            info!(
                "[Steam MP] connect_p2p dispatched to host {}",
                host_steam_id.raw()
            );
        }
        Err(_) => {
            warn!("[Steam MP] connect_p2p failed (InvalidHandle)");
            socket.expected_peer = None;
            connection.error = Some(
                "Steam couldn't open a P2P connection to your friend. Try again.".to_string(),
            );
            connection.state = ConnectionState::Failed;
        }
    }
}

/// Guest-side: poll the in-flight `NetConnection`'s real-time status each
/// frame while we're still dialling. Promotes `NetworkConnection.state` to
/// `Connected` on success, or `Disconnected` on `ProblemDetectedLocally` /
/// `ClosedByPeer`. No-op once already `Connected` (the bridge takes over).
pub(super) fn poll_steam_guest_connection_state(
    client: Option<Res<Client>>,
    mut socket: Option<ResMut<SteamP2pSocket>>,
    mut connection: ResMut<NetworkConnection>,
) {
    if connection.mode != ConnectionMode::Steam || connection.role != Some(PeerRole::Guest) {
        return;
    }
    if connection.state == ConnectionState::Connected {
        return;
    }
    let (Some(client), Some(socket)) = (client.as_deref(), socket.as_deref_mut()) else {
        return;
    };
    let Some(net_conn) = socket.connection.as_ref() else {
        return;
    };

    match client
        .networking_sockets()
        .get_realtime_connection_status(net_conn, 0)
    {
        Ok((info, _lane_status)) => match info.connection_state() {
            Ok(NetworkingConnectionState::Connected) => {
                info!("[Steam MP] Guest connection reached Connected");
                connection.state = ConnectionState::Connected;
            }
            Ok(NetworkingConnectionState::ProblemDetectedLocally)
            | Ok(NetworkingConnectionState::ClosedByPeer)
            | Ok(NetworkingConnectionState::None) => {
                warn!(
                    "[Steam MP] Guest connection failed (state={:?})",
                    info.connection_state()
                );
                tear_down_socket(socket);
                connection.state = ConnectionState::Disconnected;
            }
            // Still dialling (Connecting / FindingRoute / Linger / Dead) — try again next frame.
            Ok(_) | Err(_) => {}
        },
        Err(_) => {
            warn!("[Steam MP] get_realtime_connection_status returned an error");
            tear_down_socket(socket);
            connection.state = ConnectionState::Disconnected;
        }
    }
}

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

    // --- Send outgoing reliable -------------------------------------------
    let outgoing_msgs: Vec<NetworkMessage> = connection.outgoing_messages.drain(..).collect();
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

    // --- Send outgoing unreliable -----------------------------------------
    let outgoing_unrel: Vec<Vec<u8>> = connection.outgoing_unreliable.drain(..).collect();
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
