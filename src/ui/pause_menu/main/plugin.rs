//! Pause menu main screen plugin.

use bevy::prelude::*;

use crate::state::PauseMenuState;
use crate::ui::plugin::ButtonActionSet;
use crate::ui::systems::{escape_to_running, handle_scroll};

use crate::game::multiplayer::coop_pause::coop_local_is_pause_controller;

use super::components::ScrollablePauseStats;
use super::systems::{button_action, relabel_continue_for_coop, setup};

/// Plugin that manages the pause menu main screen UI.
///
/// Registers systems for:
/// - Pause menu main screen setup and cleanup
/// - Button interactions and visual feedback
/// - Menu navigation and state transitions
#[derive(Default)]
pub struct PauseMainPlugin;

impl Plugin for PauseMainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(PauseMenuState::Main),
            // In a co-op sync-pause, a NON-initiator's Continue button is relabeled
            // "Waiting for other player" and the resume is blocked (below).
            (setup, relabel_continue_for_coop).chain(),
        )
        .add_systems(
            OnExit(PauseMenuState::Main),
            crate::ui::systems::cleanup_screen::<super::components::OnPauseMainScreen>,
        )
        .add_systems(
            Update,
            button_action
                .in_set(ButtonActionSet)
                .run_if(in_state(PauseMenuState::Main)),
        )
        .add_systems(
            Update,
            handle_scroll::<ScrollablePauseStats>.run_if(in_state(PauseMenuState::Main)),
        )
        // Escape-to-resume only fires for the pause controller (always true in SP /
        // versus; in co-op only the initiator may resume).
        .add_systems(
            Update,
            escape_to_running
                .run_if(in_state(PauseMenuState::Main).and_then(coop_local_is_pause_controller)),
        );
    }
}
