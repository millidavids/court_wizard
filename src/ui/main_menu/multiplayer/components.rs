//! Multiplayer screen specific components.

use bevy::prelude::*;

use crate::config::WizardType;
use crate::game::units::wizard::components::Spell;

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

    /// Select a wizard type in the lobby.
    SelectWizard(WizardType),

    /// Toggle a spell in the action bar.
    ToggleSpell(Spell),

    /// Mark as ready to start the match.
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

/// Marker for the wizard select phase UI container.
#[derive(Component)]
pub(super) struct WizardSelectContainer;

/// Marker for the opponent info display text.
#[derive(Component)]
pub(super) struct OpponentInfoText;

/// Marker for the ready button container.
#[derive(Component)]
pub(super) struct ReadyButtonContainer;

/// Marker for individual wizard card in the lobby.
#[derive(Component)]
pub(super) struct LobbyWizardCard(pub WizardType);

/// Marker for a spell slot button in the lobby.
#[derive(Component)]
pub(super) struct LobbySpellSlot(pub usize);

/// Marker for the action bar display container.
#[derive(Component)]
pub(super) struct ActionBarContainer;

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
        /// This player's unlocked spells.
        my_spells: Vec<Spell>,
        /// Opponent's unlocked wizard types (for display).
        opponent_wizard_types: Vec<WizardType>,
        /// This player's selected wizard.
        my_wizard: Option<WizardType>,
        /// Opponent's selected wizard.
        opponent_wizard: Option<WizardType>,
        /// This player's action bar.
        my_action_bar: [Option<Spell>; 5],
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
