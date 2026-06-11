//! Multiplayer tab plugin — system registration only.

use bevy::prelude::*;

use crate::state::{MenuState, MetaGameState};
use crate::ui::plugin::ButtonActionSet;
use crate::ui::wizard_tower::wizard_cards::SelectedWizard;

use super::interaction::{cancel_host_on_tab_leave, handle_mp_tab_actions};
use super::lobby_messages::process_lobby_messages;
use super::state::{CoopHostSelection, MultiplayerLobby};
use super::sync::{
    broadcast_host_mode_to_guest, sync_lobby_with_connection, sync_mp_wizard_selection,
};
use super::systems::{
    handle_pending_rematch_on_enter, mp_tab_selected, reset_lobby_on_exit,
    route_pending_rematch_from_menu,
};
use super::text_input::handle_join_code_input;

/// Plugin that registers all systems for the multiplayer tab.
pub struct MultiplayerTabPlugin;

impl Plugin for MultiplayerTabPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MultiplayerLobby>()
            .init_resource::<CoopHostSelection>()
            // A rematch routes through the main menu; bounce straight back into
            // the tower so `handle_pending_rematch_on_enter` can pick it up.
            .add_systems(OnEnter(MenuState::Landing), route_pending_rematch_from_menu)
            .add_systems(
                OnEnter(MetaGameState::WizardTower),
                handle_pending_rematch_on_enter,
            )
            .add_systems(OnExit(MetaGameState::WizardTower), reset_lobby_on_exit)
            // The lobby network pump must run on any Wizard Tower tab — not just
            // the Multiplayer tab — so a connection isn't stranded if the player
            // switches tabs mid-handshake.
            .add_systems(
                Update,
                (process_lobby_messages, sync_lobby_with_connection)
                    .run_if(in_state(MetaGameState::WizardTower)),
            )
            // The host broadcasts its selected mode to the guest on ANY tab (it
            // sits on Endless/Roguelite/VS while the guest waits on Multiplayer),
            // so this is gated only on the tower + the tab resource existing.
            .add_systems(
                Update,
                broadcast_host_mode_to_guest
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(resource_exists::<crate::ui::wizard_tower::layout::WizardTowerTab>),
            )
            // Cancels a still-waiting host attempt when the host leaves the
            // multiplayer tabs (runs on any tower tab to observe the transition).
            .add_systems(
                Update,
                cancel_host_on_tab_leave
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(resource_exists::<crate::ui::wizard_tower::layout::WizardTowerTab>),
            )
            // Tab UI interaction only runs while the Multiplayer tab is shown.
            .add_systems(
                Update,
                (
                    handle_mp_tab_actions.in_set(ButtonActionSet),
                    handle_join_code_input,
                )
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(mp_tab_selected),
            )
            // The wizard-selection sync needs `SelectedWizard` to exist (it is
            // inserted lazily), so it carries its own resource-gated condition.
            .add_systems(
                Update,
                sync_mp_wizard_selection
                    .run_if(in_state(MetaGameState::WizardTower))
                    .run_if(mp_tab_selected)
                    .run_if(
                        resource_exists::<SelectedWizard>.and(resource_changed::<SelectedWizard>),
                    ),
            );
    }
}
