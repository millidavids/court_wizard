use bevy::prelude::*;

use crate::game::run_conditions;
use crate::state::{AppState, InGameState};

use super::systems;

/// Plugin managing the rune display UI.
pub struct RuneDisplayPlugin;

impl Plugin for RuneDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), systems::spawn_rune_display)
            .add_systems(
                OnEnter(InGameState::Running),
                systems::spawn_rune_display.run_if(run_conditions::coming_from_game_over),
            )
            .add_systems(
                Update,
                systems::update_rune_display.run_if(in_state(InGameState::Running)),
            );
    }
}
