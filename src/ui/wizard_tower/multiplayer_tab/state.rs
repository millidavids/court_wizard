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
                .filter(|wt| names.iter().any(|n| n == &format!("{:?}", wt)))
                .collect::<Vec<_>>()
        })
        .filter(|wts: &Vec<_>| !wts.is_empty())
        .unwrap_or_else(|| vec![WizardType::BoringOleMage]);
    (wizard_types, Spell::all().to_vec())
}
