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

    /// Host tells guest to start loading the match, sharing the run seed so
    /// both peers seed their RNG identically.
    StartGame { seed: u64 },

    /// Player has finished loading.
    GameLoaded,

    /// Host notifies guest that the game is over.
    GameOver(GameOverResult),

    /// Player is ready for a rematch (sent from score screen).
    RematchReady,

    /// A wall was placed or removed — update pathfinding grid.
    ///
    /// Sent bidirectionally when either player places a Wall of Dirt so
    /// the other client can update its pathfinding grid.
    WallPlaced {
        /// AABB of the wall obstacle: [min_x, min_z, max_x, max_z].
        bounds: [f32; 4],
        /// true = wall placed (Blocked), false = wall removed (Removed).
        placed: bool,
    },

    /// Guest tells the host to teleport units within a radius.
    ///
    /// Unit positions are host-authoritative, so the guest sends this message
    /// instead of moving units locally.
    TeleportUnits {
        source_x: f32,
        source_z: f32,
        dest_x: f32,
        dest_z: f32,
        radius: f32,
    },

    /// Guest tells the host that one of its spells just impacted a unit.
    ///
    /// Status-effect bookkeeping is host-authoritative: the host receives this
    /// message, inserts the standard `PendingDamageEffect` on the matching
    /// local entity (resolved through `NetworkEntityMap`), and SP's existing
    /// `process_pending_damage_effects` pipeline takes over from there —
    /// FireDoT / FrostAccumulation / Shocked / Poisoned all flow through that
    /// shared code path on the host. The host's normal snapshot then ships
    /// the resulting status flag (e.g. `UnitFlags::FIRE_EFFECT`) back to the
    /// guest, which renders the visual via `RemoteFireEffect` etc.
    SpellHitUnit {
        target_network_id: u32,
        damage: f32,
        /// Serialized `DamageType` ordinal — see `DamageType::from_u8`.
        damage_type: u8,
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
