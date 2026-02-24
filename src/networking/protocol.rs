//! Network message protocol.
//!
//! Defines the serializable message types sent over the WebRTC data channels.
//! Reliable channel: commands, events, and setup messages.
//! Unreliable channel: raw binary state snapshots (handled separately).

use serde::{Deserialize, Serialize};

use crate::config::WizardType;
use crate::game::units::wizard::components::Spell;

/// Messages sent over the reliable WebRTC data channel between peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Ping with a timestamp for RTT measurement.
    Ping { timestamp_ms: f64 },

    /// Pong response echoing back the original timestamp.
    Pong { timestamp_ms: f64 },

    /// Player info exchanged after connection (unlocked content).
    PlayerInfo {
        wizard_types: Vec<WizardType>,
        spells: Vec<Spell>,
    },

    /// Player selected a wizard type.
    WizardSelected(WizardType),

    /// Player is ready to start the match.
    ReadyUp,

    /// Player cancelled their ready state.
    Unready,

    /// Host tells guest to start loading the match.
    StartGame,

    /// Player has finished loading.
    GameLoaded,

    /// Host notifies guest that the game is over.
    GameOver(GameOverResult),

    /// Player is ready for a rematch (sent from score screen).
    RematchReady,

    /// A wall was placed or removed — update pathfinding grid.
    ///
    /// Sent bidirectionally when either player places a Wall of Stone so
    /// the other client can update its pathfinding grid.
    WallPlaced {
        /// AABB of the wall obstacle: [min_x, min_z, width, height].
        bounds: [f32; 4],
        /// true = wall placed (Blocked), false = wall removed (Removed).
        placed: bool,
    },
}

/// Result of a multiplayer match.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GameOverResult {
    /// The host won the match.
    HostWins,

    /// The guest won the match.
    GuestWins,
}
