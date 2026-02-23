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

    /// Guest sends a spell result to the host for spawning.
    SpellResult(SpellAction),

    /// Host notifies guest that the game is over.
    GameOver(GameOverResult),

    /// Player is ready for a rematch (sent from score screen).
    RematchReady,
}

/// Spell result actions sent from the guest to the host.
///
/// Each client runs its own casting pipeline (cast time, mana, cast bar).
/// When a spell completes, the client sends the result with only the data
/// the host needs to spawn the spell effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpellAction {
    /// Fire-and-forget spell completed casting.
    SpellCast {
        spell: Spell,
        cursor_pos: [f32; 3],
        empowerment: f32,
    },

    /// Drag-to-place spell completed (wall of stone, wall of fire).
    DragSpellCast {
        spell: Spell,
        anchor: [f32; 3],
        end_pos: [f32; 3],
        empowerment: f32,
    },

    /// Channeled spell started (cast time completed, entering channeling).
    ChannelStarted {
        spell: Spell,
        cursor_pos: [f32; 3],
        empowerment: f32,
    },

    /// Channeled spell cursor update (each frame during channeling).
    ChannelUpdate { cursor_pos: [f32; 3] },

    /// Channeled spell ended (mouse released or mana ran out).
    ChannelEnded,
}

/// Result of a multiplayer match.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GameOverResult {
    /// The host won the match.
    HostWins,

    /// The guest won the match.
    GuestWins,
}
