//! State types for the multiplayer lobby tab.

use bevy::prelude::*;

use crate::config::WizardType;
use crate::config::save_data;
use crate::game::units::wizard::components::Spell;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Tracks the current state of the multiplayer lobby tab.
///
/// This is the single source of truth for which phase the lobby is in,
/// plus transient UI state (relay toggle, join-code text input).
#[derive(Resource, PartialEq)]
pub(crate) struct MultiplayerLobby {
    /// Current phase of the lobby flow.
    pub phase: LobbyPhase,
    /// Whether to use relay servers for NAT traversal (default: true).
    pub use_relay: bool,
    /// Text the guest is typing for the host's ticket code.
    pub join_code_input: String,
    /// Whether the join code text field is focused for keyboard input.
    pub join_code_focused: bool,
    /// Transient feedback line shown in the right panel (e.g. "Code copied!").
    pub status_message: Option<String>,
    /// Wire-protocol version received from the peer's `HandshakeVersion`.
    /// `None` until the peer sends one. The lobby refuses to process
    /// `PlayerInfo` (or any other message that advances the phase) until
    /// this is set to a matching version. An old-binary peer never sends
    /// `HandshakeVersion`, so its first `PlayerInfo` arrives with this
    /// still `None` → fast-fail with a version-mismatch error.
    pub peer_protocol_version: Option<u32>,
}

impl MultiplayerLobby {
    pub fn new() -> Self {
        Self {
            phase: LobbyPhase::default(),
            // Default to Online (relay) — works for friends anywhere, not just LAN.
            use_relay: true,
            join_code_input: String::new(),
            join_code_focused: false,
            status_message: None,
            peer_protocol_version: None,
        }
    }
}

impl Default for MultiplayerLobby {
    fn default() -> Self {
        Self::new()
    }
}

/// Guest-side mirror of the host's currently-selected game mode in the lobby.
/// Populated from `NetworkMessage::HostModeSelection` (received in
/// `process_lobby_messages`) and rendered in the guest's Multiplayer-tab LEFT
/// panel so the guest sees what the host is about to start. Host-only fields are
/// pre-formatted into `detail_lines` so the guest needs no game-mode resources.
#[derive(Resource, Default, PartialEq, Clone)]
pub(crate) struct CoopHostSelection {
    pub mode: crate::networking::protocol::HostMode,
    pub host_wizard: Option<WizardType>,
    pub level: u32,
    pub is_continue: bool,
    pub detail_lines: Vec<String>,
}

// ---------------------------------------------------------------------------
// LobbyPhase
// ---------------------------------------------------------------------------

/// Phase of the multiplayer lobby flow.
#[derive(Default, PartialEq, Clone)]
pub(crate) enum LobbyPhase {
    /// Initial screen — show Host Game, Join Game, Use Relay toggle.
    #[default]
    Connect,

    /// Hosting: local ticket code generated, waiting for guest to connect.
    Hosting,

    /// Joining: guest is entering the host's ticket code.
    Joining,

    /// Steam hosting: lobby created, friends overlay opened, waiting for the
    /// invited friend to accept and connect over SDR.
    SteamHosting,

    /// Steam joining: lobby joined (via overlay invite, friend-list Join Game,
    /// or `+connect_lobby` launch param), waiting for SDR connection.
    SteamJoining,

    /// Transport connected, exchanging PlayerInfo with opponent.
    Handshake,

    /// Both players have exchanged info — selecting wizards, ready-up.
    WizardSelect {
        /// This player's unlocked wizard types.
        my_wizard_types: Vec<WizardType>,
        /// Opponent's unlocked wizard types.
        opponent_wizard_types: Vec<WizardType>,
        /// This player's currently selected wizard.
        my_wizard: Option<WizardType>,
        /// Opponent's currently selected wizard.
        opponent_wizard: Option<WizardType>,
        /// Whether this player has clicked Ready.
        my_ready: bool,
        /// Whether the opponent has clicked Ready.
        opponent_ready: bool,
    },

    /// Connection failed — show error and offer Retry/Back.
    Failed { reason: String },
}

// ---------------------------------------------------------------------------
// Button action component
// ---------------------------------------------------------------------------

/// Button actions specific to the multiplayer tab.
#[derive(Component, Debug, Clone, PartialEq)]
pub(crate) enum MpTabAction {
    /// Start hosting a game (create host endpoint).
    HostGame,
    /// Switch to the Joining phase (show text input).
    JoinGame,
    /// Create a Steam lobby and open the friends invite overlay.
    SteamInvite,
    /// Toggle the "Use relay" setting.
    ToggleRelay,
    /// Copy the local ticket code to the clipboard.
    CopyCode,
    /// Paste text from the clipboard into the join-code field.
    PasteFromClipboard,
    /// Connect to the host using the current join_code_input.
    ConfirmJoin,
    /// Cancel the current operation and return to Connect phase.
    Cancel,
    /// Retry after a failure (return to Connect phase, reset connection).
    Retry,
    /// Disconnect and return to Connect phase.
    Disconnect,
    /// Mark this player as ready to start the match.
    Ready,
    /// Unmark ready.
    Unready,
    /// Open the shared wizard-card grid to switch wizard.
    SwitchWizard,
    /// Host only: start the match (enabled once both players are ready).
    StartGame,
}

// ---------------------------------------------------------------------------
// Marker components for reactive panel updates
// ---------------------------------------------------------------------------

/// Marker on the join-code text display node.
#[derive(Component)]
pub(crate) struct JoinCodeInputDisplay;

/// Marker on the join-code input box (clickable to focus).
#[derive(Component)]
pub(crate) struct JoinCodeInputBox;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Loads this player's unlocked wizard types and all spells from save data.
pub(crate) fn load_my_unlocked_content() -> (Vec<WizardType>, Vec<Spell>) {
    let wizard_types = save_data::load_unified_save()
        .map(|save| {
            let names = save.player.unlocked_content.wizard_types;
            WizardType::all()
                .iter()
                .copied()
                // Psychopath is disabled in multiplayer — its self-sabotage win
                // condition (kill 70% of your own defenders) doesn't map to a
                // competitive match. Filtering here keeps it out of the lobby grid
                // AND the `PlayerInfo` exchange, so it can never reach a match.
                .filter(|wt| *wt != WizardType::Psychopath)
                .filter(|wt| names.iter().any(|n| n == &format!("{:?}", wt)))
                .collect::<Vec<_>>()
        })
        .filter(|wts: &Vec<_>| !wts.is_empty())
        .unwrap_or_else(|| vec![WizardType::BoringOleMage]);
    (wizard_types, Spell::all().to_vec())
}

/// Returns the connected co-op partner's chosen wizard IF this peer is the HOST
/// and connected (the lobby reached wizard-select). Used by the Endless/Roguelite
/// tabs to launch a co-op match when the host starts a game with a guest present.
pub(crate) fn connected_coop_guest_wizard(
    connection: &crate::networking::resources::NetworkConnection,
    lobby: Option<&MultiplayerLobby>,
) -> Option<WizardType> {
    use crate::networking::resources::{ConnectionState, PeerRole};
    if connection.state != ConnectionState::Connected || connection.role != Some(PeerRole::Host) {
        return None;
    }
    match lobby?.phase {
        LobbyPhase::WizardSelect {
            opponent_wizard: Some(w),
            ..
        } => Some(w),
        _ => None,
    }
}
