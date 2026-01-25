//! Input plugin for centralizing input detection.

use bevy::prelude::*;

use crate::state::InGameState;

use super::{events::*, systems};

/// Plugin that handles all game input detection.
///
/// Queries input state once per frame and sends events that other
/// systems can consume, avoiding duplicate input queries.
#[derive(Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
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
            // Add input detection systems
            .add_systems(
                Update,
                (systems::detect_mouse_input, systems::detect_keyboard_input)
                    .run_if(in_state(InGameState::Running)),
            );
    }
}
