//! Input plugin for centralizing input detection.

use bevy::prelude::*;

use crate::game::run_conditions;
use crate::game::run_conditions::is_gameplay_running;
use crate::state::InGameState;

use super::{
    components::{
        MouseButtonState, MouseLeftHeldThisFrame, MouseRightHeldThisFrame,
        SpellInputBlockedThisFrame,
    },
    messages::*,
    systems,
};

/// Plugin that handles all game input detection.
///
/// Queries input state once per frame and sends events that other
/// systems can consume, avoiding duplicate input queries.
#[derive(Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize input resources
            .init_resource::<MouseButtonState>()
            .init_resource::<SpellInputBlockedThisFrame>()
            .init_resource::<MouseLeftHeldThisFrame>()
            .init_resource::<MouseRightHeldThisFrame>()
            // Register input events
            .add_message::<MouseLeftPressed>()
            .add_message::<MouseLeftHeld>()
            .add_message::<MouseLeftReleased>()
            .add_message::<MouseRightPressed>()
            .add_message::<MouseRightHeld>()
            .add_message::<MouseRightReleased>()
            .add_message::<SpacebarPressed>()
            .add_message::<SpacebarHeld>()
            .add_message::<SpacebarReleased>()
            .add_message::<BlockSpellInput>()
            .add_message::<MouseClicked>()
            .add_message::<ActionBarKeyPressed>()
            // Clear stale mouse state on entering gameplay
            .add_systems(
                OnEnter(InGameState::Running),
                systems::clear_mouse_input_state,
            )
            // Add input detection systems
            .add_systems(
                Update,
                (
                    systems::detect_mouse_input,
                    systems::update_input_state_for_run_conditions,
                )
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                Update,
                systems::detect_keyboard_input
                    .run_if(is_gameplay_running.or(in_state(InGameState::SpellBook))),
            )
            // Rune input (Q/W/E/R + spacebar activate) - RuneCaster only
            .add_systems(
                Update,
                systems::detect_rune_input
                    .run_if(is_gameplay_running.or(in_state(InGameState::SpellBook)))
                    .run_if(run_conditions::is_rune_caster),
            )
            // Roulette input (spacebar to spin) - Randomancer only
            .add_systems(
                Update,
                systems::detect_roulette_input
                    .run_if(is_gameplay_running)
                    .run_if(run_conditions::is_randomancer),
            );
    }
}
