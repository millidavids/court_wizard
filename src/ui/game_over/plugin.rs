use bevy::prelude::*;

use crate::state::InGameState;
use crate::ui::plugin::ButtonActionSet;

use crate::game::achievements::systems::send_battle_ended;

use super::systems::*;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InGameState::ScoreScreen),
            (
                send_battle_ended,
                save_efficiency_to_config,
                save_walls_on_victory,
                setup_game_over_screen,
                update_level_after_display,
            )
                .chain(),
        )
        .add_systems(
            OnExit(InGameState::ScoreScreen),
            crate::ui::systems::cleanup_screen::<super::components::OnGameOverScreen>,
        )
        .add_systems(
            Update,
            handle_button_actions
                .in_set(ButtonActionSet)
                .run_if(in_state(InGameState::ScoreScreen)),
        );
    }
}
