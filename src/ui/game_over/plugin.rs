use bevy::prelude::*;

use crate::state::InGameState;

use super::systems::*;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InGameState::GameOver),
            (
                save_efficiency_to_config,
                setup_game_over_screen,
                update_level_after_display,
            )
                .chain(),
        )
        .add_systems(OnExit(InGameState::GameOver), cleanup_game_over_screen)
        .add_systems(
            Update,
            handle_button_actions.run_if(in_state(InGameState::GameOver)),
        );
    }
}
