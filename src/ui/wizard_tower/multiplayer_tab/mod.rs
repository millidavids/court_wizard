//! Multiplayer tab for the Wizard Tower — host/join lobby, wizard select, ready-up.

pub(super) mod host_mode_broadcast;
pub(super) mod interaction;
pub(super) mod lobby_messages;
pub(super) mod panel_connect;
pub(super) mod panel_failed;
pub(super) mod panel_guest_mirror;
pub(super) mod panel_handshake;
pub(super) mod panel_hosting;
pub(super) mod panel_joining;
pub(super) mod panel_steam_hosting;
pub(super) mod panel_steam_joining;
pub(super) mod panel_styles;
pub(super) mod panel_wizard_select;
pub(super) mod panels;
pub(super) mod plugin;
pub(super) mod state;
pub(super) mod sync;
pub(super) mod systems;
pub(super) mod text_input;
pub(super) mod timeout;

pub(crate) use plugin::MultiplayerTabPlugin;
pub(crate) use state::{CoopHostSelection, MultiplayerLobby};
// Test-only: the `session_reset` regression test names a phase from outside the
// tab module. All production `LobbyPhase` consumers live inside it.
#[cfg(test)]
pub(crate) use state::LobbyPhase;
pub(crate) use systems::force_mp_tab;
