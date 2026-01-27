use bevy::prelude::*;

use crate::state::InGameState;

/// Run condition that returns true only when transitioning from GameOver to Running.
///
/// This is used to ensure setup systems only run when replaying the game,
/// not when transitioning from other states like SpellBook or Paused.
pub fn coming_from_game_over(
    mut transitions: MessageReader<StateTransitionEvent<InGameState>>,
) -> bool {
    transitions.read().any(|transition| {
        transition.exited == Some(InGameState::GameOver)
            && transition.entered == Some(InGameState::Running)
    })
}
