//! Multiplayer screen specific components.
//!
//! Shared components (DetailName, DetailDescription, DetailStatus, WizardCard,
//! SelectedWizardPreview) live in `wizard_select_shared`.

use bevy::prelude::*;

use crate::config::WizardType;
use crate::networking::resources::PeerRole;

/// Marker component for entities that belong to the multiplayer screen.
///
/// Used for cleanup when exiting the multiplayer state.
#[derive(Component)]
pub(super) struct OnMultiplayerScreen;

/// Actions that can be triggered by multiplayer screen buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MultiplayerButtonAction {
    /// Start hosting a game (create offer).
    HostGame,

    /// Join a game (paste host's invite code).
    JoinGame,

    /// Copy the local signaling code to clipboard.
    CopyCode,

    /// Paste the remote peer's response code.
    PasteResponse,

    /// Disconnect and return to the main menu.
    Disconnect,

    /// Cancel the current connection attempt and return to the base multiplayer screen.
    Cancel,

    /// Retry connection after a failure (regenerate SDP/ICE).
    Retry,

    /// Return to the main menu landing screen.
    Back,

    /// Preview a wizard type (show in detail panel).
    PreviewWizard(WizardType),

    /// Mark as ready to start the match with the previewed wizard.
    Ready,

    /// Cancel ready state.
    Unready,

    /// Start hosting a LAN game (transitions to IP entry phase).
    LanHost,

    /// Start joining a LAN game (transitions to IP entry phase).
    LanJoin,

    /// Confirm the entered IP and proceed to LAN signaling.
    LanConfirmIp,

    /// Open a prompt to enter or change the local IP address.
    LanEditIp,

    /// Cancel LAN IP entry and return to the initial screen.
    LanIpCancel,
}

/// Marker for the ready/unready button container in the detail panel.
/// Stores the last displayed ready state to avoid rebuilding every frame.
#[derive(Component)]
pub(super) struct ReadyButtonArea {
    pub showing_ready: bool,
}

/// Marker component for the status text that updates based on connection state.
#[derive(Component)]
pub(super) struct StatusText;

/// Marker component for the code display text area.
#[derive(Component)]
pub(super) struct CodeDisplayText;

/// Marker component for the ping display text.
#[derive(Component)]
pub(super) struct PingText;

/// Marker for the initial button group (Host Game / Join Game).
#[derive(Component)]
pub(super) struct InitialButtons;

/// Marker for the signaling button group (Paste Response / Cancel).
#[derive(Component)]
pub(super) struct SignalingButtons;

/// Marker for the Paste Response button (only visible for host).
#[derive(Component)]
pub(super) struct PasteResponseButton;

/// Marker for the Copy Code button (lives in the right info column).
#[derive(Component)]
pub(super) struct CopyCodeButton;

/// Marker for the connecting/connected/failed button group (Cancel / Disconnect / Try Again).
#[derive(Component)]
pub(super) struct ActiveConnectionButtons;

/// Marker for the LAN button group and its section label.
#[derive(Component)]
pub(super) struct LanButtons;

/// Marker for the Back button (hidden during signaling/active connection).
#[derive(Component)]
pub(super) struct BackButton;

/// Marker for the LAN IP entry button group (Change IP / Confirm / Cancel).
#[derive(Component)]
pub(super) struct LanIpEntryButtons;

/// Marker for the IP display text in the right column during LAN IP entry.
#[derive(Component)]
pub(super) struct IpDisplayText;

/// Marker for the page title text (dynamically updated to show mode/role).
#[derive(Component)]
pub(super) struct TitleText;

/// Marker for the wizard select phase container (full screen layout).
#[derive(Component)]
pub(super) struct WizardSelectScreen;

/// Tracks the current phase of the multiplayer lobby.
#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub(super) enum LobbyPhase {
    /// Initial connection phase — showing Host/Join buttons.
    #[default]
    Connection,

    /// LAN IP entry phase (legacy — ticket-based connection supersedes this).
    #[allow(dead_code)]
    LanIpEntry {
        /// Whether the user intends to host or join.
        role: PeerRole,
        /// The currently entered/displayed IP (from saved or user entry).
        current_ip: Option<String>,
    },

    /// Connected, waiting for PlayerInfo exchange.
    WaitingForPlayerInfo,

    /// Both players have exchanged info, selecting wizards.
    WizardSelect {
        /// This player's unlocked wizard types.
        my_wizard_types: Vec<WizardType>,
        /// Opponent's unlocked wizard types (for display).
        opponent_wizard_types: Vec<WizardType>,
        /// This player's selected wizard.
        my_wizard: Option<WizardType>,
        /// Opponent's selected wizard.
        opponent_wizard: Option<WizardType>,
        /// Whether this player is ready.
        my_ready: bool,
        /// Whether the opponent is ready.
        opponent_ready: bool,
    },
}
