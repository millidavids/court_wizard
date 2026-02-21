//! Networking plugin.
//!
//! Registers the networking resource and systems for WebRTC P2P communication.

use bevy::prelude::*;

use super::constants::PING_INTERVAL_SECS;
use super::protocol::NetworkMessage;
use super::resources::{ConnectionState, NetworkConnection, PeerRole};

#[cfg(target_arch = "wasm32")]
use super::resources::ConnectionMode;

/// Plugin that manages P2P networking via WebRTC.
///
/// On WASM targets, this syncs the thread-local WebRTC state into the ECS each frame
/// and handles ping/pong for latency measurement. On native targets, the resource
/// exists but no WebRTC systems run.
#[derive(Default)]
pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkConnection>();

        #[cfg(target_arch = "wasm32")]
        {
            app.add_systems(
                Update,
                (
                    super::webrtc::sync_webrtc_state,
                    poll_lan_signaling,
                    send_outgoing_messages,
                    send_outgoing_unreliable,
                    send_ping,
                    handle_incoming_messages,
                )
                    .chain()
                    .run_if(is_networking_active),
            );
        }
    }
}

/// Run condition: returns true when the network connection is not disconnected.
fn is_networking_active(connection: Res<NetworkConnection>) -> bool {
    connection.state != ConnectionState::Disconnected
}

/// Polls the BroadcastChannel for LAN auto-signaling.
///
/// In LAN mode, automatically exchanges offer/answer codes between tabs on
/// the same machine. For different LAN machines, this is a no-op and the
/// manual copy-paste fallback is used.
#[cfg(target_arch = "wasm32")]
fn poll_lan_signaling(connection: Res<NetworkConnection>) {
    if connection.mode != ConnectionMode::Lan {
        return;
    }

    // HOST: broadcast offer when local_code ready, check for auto-received answers
    if connection.role == Some(PeerRole::Host)
        && connection.state == ConnectionState::WaitingForSignaling
        && connection.local_code.is_some()
    {
        super::lan_signaling::broadcast_offer(connection.local_code.as_ref().unwrap());
        if let Some(answer) = super::lan_signaling::take_received_answer() {
            super::webrtc::process_answer(&answer);
        }
    }

    // GUEST: check for auto-received offers (before we have our own code)
    if connection.role == Some(PeerRole::Guest)
        && connection.state == ConnectionState::WaitingForSignaling
        && connection.local_code.is_none()
    {
        if let Some(offer) = super::lan_signaling::take_received_offer() {
            super::webrtc::create_guest_answer(&offer, false);
        }
    }

    // GUEST: broadcast answer when local_code ready
    if connection.role == Some(PeerRole::Guest)
        && connection.state == ConnectionState::WaitingForSignaling
        && connection.local_code.is_some()
    {
        super::lan_signaling::broadcast_answer(connection.local_code.as_ref().unwrap());
    }
}

/// Serializes and sends all queued outgoing messages over the data channel.
#[cfg(target_arch = "wasm32")]
fn send_outgoing_messages(mut connection: ResMut<NetworkConnection>) {
    if connection.state != ConnectionState::Connected {
        return;
    }

    let messages: Vec<NetworkMessage> = connection.outgoing_messages.drain(..).collect();
    for msg in &messages {
        if let Ok(json) = serde_json::to_string(msg) {
            super::webrtc::send_message(&json);
        }
    }
}

/// Sends all queued outgoing unreliable binary data over the unreliable data channel.
#[cfg(target_arch = "wasm32")]
fn send_outgoing_unreliable(mut connection: ResMut<NetworkConnection>) {
    if connection.state != ConnectionState::Connected {
        return;
    }

    let data: Vec<Vec<u8>> = connection.outgoing_unreliable.drain(..).collect();
    for bytes in &data {
        super::webrtc::send_unreliable(bytes);
    }
}

/// Sends a ping message at regular intervals when connected.
#[cfg(target_arch = "wasm32")]
fn send_ping(mut connection: ResMut<NetworkConnection>, time: Res<Time>) {
    if connection.state != ConnectionState::Connected {
        return;
    }

    connection.ping_timer += time.delta_secs();
    if connection.ping_timer >= PING_INTERVAL_SECS {
        connection.ping_timer = 0.0;
        let timestamp_ms = js_sys::Date::now();
        connection
            .outgoing_messages
            .push(NetworkMessage::Ping { timestamp_ms });
    }
}

/// Processes incoming messages, handling ping/pong internally.
///
/// Game-specific messages (PlayerInfo, SpellCommand, etc.) are left in the
/// incoming queue for game systems to process. Only networking-level messages
/// (Ping/Pong) are consumed here.
///
/// Uses a read-only check first to avoid triggering Bevy change detection
/// on frames with no messages (which would cause UI rebuilds making buttons unclickable).
#[cfg(target_arch = "wasm32")]
fn handle_incoming_messages(connection: ResMut<NetworkConnection>) {
    if connection.incoming_messages.is_empty() {
        return;
    }

    let mut connection = connection;
    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();

    let mut game_messages = Vec::new();

    for msg in messages {
        match &msg {
            NetworkMessage::Ping { timestamp_ms } => {
                connection
                    .outgoing_messages
                    .push(NetworkMessage::Pong {
                        timestamp_ms: *timestamp_ms,
                    });
            }
            NetworkMessage::Pong { timestamp_ms } => {
                let now = js_sys::Date::now();
                let rtt = (now - timestamp_ms) as f32;
                connection.ping_ms = Some(rtt);
            }
            // All other messages are game-level; put them back for game systems.
            _ => {
                game_messages.push(msg);
            }
        }
    }

    if !game_messages.is_empty() {
        connection.incoming_messages = game_messages;
    }
}
