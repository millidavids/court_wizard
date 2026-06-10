//! Multiplayer session resource and run conditions.

use bevy::prelude::*;

use super::resources::PeerRole;
use crate::config::WizardType;
use crate::game::units::wizard::components::Spell;

/// Which flavor of multiplayer match this session is.
///
/// `Versus` is the original 1v1 duel (both peers in `AppState::MultiplayerGame`).
/// The `Coop*` variants are cooperative play on the single-player endless/
/// roguelite battlefield: the host runs the authoritative simulation in
/// `AppState::InGame` while the guest spectates+casts in `MultiplayerGame`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionMode {
    Versus,
    CoopEndless,
    CoopRoguelite,
}

impl SessionMode {
    /// Wire ordinal for `NetworkMessage::StartGame.coop_mode` (`None` = versus).
    pub fn to_coop_wire(self) -> Option<u8> {
        match self {
            SessionMode::Versus => None,
            SessionMode::CoopEndless => Some(0),
            SessionMode::CoopRoguelite => Some(1),
        }
    }

    /// Decode a `StartGame.coop_mode` payload back into a `SessionMode`.
    ///
    /// `None` is versus; an unrecognized `Some(n)` also falls back to `Versus`.
    /// That fallback is safe because the `PROTOCOL_VERSION` handshake rejects any
    /// cross-version peer before a `StartGame` is ever exchanged, so a `Some(n)`
    /// with an ordinal this build doesn't know can't reach here in practice — any
    /// future co-op variant must bump `PROTOCOL_VERSION`.
    pub fn from_coop_wire(wire: Option<u8>) -> Self {
        match wire {
            Some(0) => SessionMode::CoopEndless,
            Some(1) => SessionMode::CoopRoguelite,
            _ => SessionMode::Versus,
        }
    }
}

/// Tracks the active multiplayer session configuration.
///
/// Inserted when both players are ready to start a match.
/// Contains all data needed to set up the multiplayer battlefield.
#[derive(Resource)]
pub struct MultiplayerSession {
    /// Whether this peer is the host or guest.
    pub role: PeerRole,

    /// Whether this is a versus duel or a co-op endless/roguelite match.
    pub mode: SessionMode,

    /// Wizard type chosen by the host.
    pub host_wizard: WizardType,

    /// Wizard type chosen by the guest.
    pub guest_wizard: WizardType,

    /// Spells available to the host (from their unlocked spells).
    #[allow(dead_code)]
    pub host_spells: Vec<Spell>,

    /// Spells available to the guest (from their unlocked spells).
    #[allow(dead_code)]
    pub guest_spells: Vec<Spell>,

    /// True for a co-op roguelite match with the Urgent toggle active. Disables
    /// synchronized pause (each peer pauses locally without freezing the sim or
    /// the other player), matching Urgent's "game keeps running" semantics. Set
    /// from `ActiveToggles` on the host and from `StartGame.urgent` on the guest;
    /// always `false` in versus and co-op endless.
    pub coop_urgent: bool,
}

impl MultiplayerSession {
    /// This peer's own wizard type.
    pub fn local_wizard(&self) -> WizardType {
        match self.role {
            PeerRole::Host => self.host_wizard,
            PeerRole::Guest => self.guest_wizard,
        }
    }

    /// The opponent's wizard type from this peer's perspective.
    pub fn remote_wizard(&self) -> WizardType {
        match self.role {
            PeerRole::Host => self.guest_wizard,
            PeerRole::Guest => self.host_wizard,
        }
    }

    /// True for a cooperative endless/roguelite session (not a versus duel).
    pub fn is_coop(&self) -> bool {
        matches!(
            self.mode,
            SessionMode::CoopEndless | SessionMode::CoopRoguelite
        )
    }

    /// True when synchronized pause applies: a co-op match without the Urgent
    /// toggle (Urgent keeps the game running, so pause stays local there).
    pub fn coop_pause_synced(&self) -> bool {
        self.is_coop() && !self.coop_urgent
    }
}

/// Returns true when in a multiplayer game as the host.
pub fn is_multiplayer_host(session: Option<Res<MultiplayerSession>>) -> bool {
    session.is_some_and(|s| s.role == PeerRole::Host)
}

/// Returns true when in a multiplayer game as the guest.
pub fn is_multiplayer_guest(session: Option<Res<MultiplayerSession>>) -> bool {
    session.is_some_and(|s| s.role == PeerRole::Guest)
}

/// Returns true when the active session is a co-op (endless/roguelite) match.
/// Wired as a run condition for the co-op attacker-effectiveness buff.
pub fn is_coop_session(session: Option<Res<MultiplayerSession>>) -> bool {
    session.is_some_and(|s| s.is_coop())
}
