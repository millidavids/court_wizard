//! Multiplayer screen specific components.
//!
//! Shared components (DetailName, DetailDescription, DetailStatus, WizardCard,
//! SelectedWizardPreview) live in `wizard_select_shared`.

use bevy::prelude::*;

use crate::config::WizardType;

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

    /// Disconnect and return to initial multiplayer screen.
    Disconnect,

    /// Return to the landing screen.
    Back,

    /// Preview a wizard type (show in detail panel).
    PreviewWizard(WizardType),

    /// Mark as ready to start the match with the previewed wizard.
    Ready,
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

/// Marker for the signaling button group (Copy Code / Paste Response / Cancel).
#[derive(Component)]
pub(super) struct SignalingButtons;

/// Marker for the Paste Response button (only visible for host).
#[derive(Component)]
pub(super) struct PasteResponseButton;

/// Marker for the connecting/connected/failed button group (Cancel / Disconnect / Try Again).
#[derive(Component)]
pub(super) struct ActiveConnectionButtons;

/// Marker for the wizard select phase container (full screen layout).
#[derive(Component)]
pub(super) struct WizardSelectScreen;

/// Tracks the current phase of the multiplayer lobby.
#[derive(Resource, Debug, Clone, PartialEq)]
pub(super) enum LobbyPhase {
    /// Initial connection phase — showing Host/Join buttons.
    Connection,

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

impl Default for LobbyPhase {
    fn default() -> Self {
        Self::Connection
    }
}
