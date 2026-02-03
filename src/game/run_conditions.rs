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

/// Check if any entities with the specified component exist.
/// Used to avoid running systems when there are no relevant entities.
///
/// Example: `any_exist::<MagicMissile>()` will only return true if there are magic missiles in the world.
///
/// This is more efficient than running systems with empty queries every frame.
pub fn any_exist<T: Component>() -> impl Fn(Query<(), With<T>>) -> bool {
    |query: Query<(), With<T>>| !query.is_empty()
}
