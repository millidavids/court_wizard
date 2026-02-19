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

    /// Player configured their action bar.
    ActionBarSet([Option<Spell>; 5]),

    /// Player is ready to start the match.
    ReadyUp,

    /// Host tells guest to start loading the match.
    StartGame,

    /// Player has finished loading.
    GameLoaded,

    /// Guest sends a spell command to the host for simulation.
    SpellCommand(SpellCommand),

    /// Host notifies guest that the game is over.
    GameOver(GameOverResult),
}

/// A spell command sent from the guest to the host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellCommand {
    pub action: SpellAction,
}

/// Specific spell input actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpellAction {
    /// Start casting a spell at the given world position.
    StartCast {
        spell: Spell,
        cursor_pos: [f32; 3],
    },

    /// Continue a channeled/drag spell at updated cursor position.
    ContinueCast { cursor_pos: [f32; 3] },

    /// Release the current spell cast.
    ReleaseCast,

    /// Prime/select a spell in the action bar.
    PrimeSpell { spell: Spell },
}

/// Result of a multiplayer match.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GameOverResult {
    /// The host won the match.
    HostWins,

    /// The guest won the match.
    GuestWins,
}
