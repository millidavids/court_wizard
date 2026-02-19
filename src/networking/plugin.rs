//! Networking plugin.
//!
//! Registers the networking resource and systems for WebRTC P2P communication.

use bevy::prelude::*;

use super::constants::PING_INTERVAL_SECS;
use super::protocol::NetworkMessage;
use super::resources::{ConnectionState, NetworkConnection};

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
#[cfg(target_arch = "wasm32")]
fn handle_incoming_messages(mut connection: ResMut<NetworkConnection>) {
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
