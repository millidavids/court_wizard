use bevy::prelude::*;

use crate::state::InGameState;

use super::resources::GameOutcome;
use super::units::components::{Corpse, Team};

/// Checks win/lose conditions every frame and transitions to GameOver state.
///
/// Win: All Attackers AND Undead are dead (only Defenders remain)
/// Lose: All Defenders are dead (Attackers or Undead remain)
pub fn check_win_lose_conditions(
    mut next_state: ResMut<NextState<InGameState>>,
    mut game_outcome: ResMut<GameOutcome>,
    units: Query<&Team, Without<Corpse>>,
) {
    let mut defenders_alive = 0;
    let mut attackers_alive = 0;
    let mut undead_alive = 0;

    for team in units.iter() {
        match team {
            Team::Defenders => defenders_alive += 1,
            Team::Attackers => attackers_alive += 1,
            Team::Undead => undead_alive += 1,
        }
    }

    // Check lose condition: no defenders left
    if defenders_alive == 0 {
        *game_outcome = GameOutcome::Defeat;
        next_state.set(InGameState::GameOver);
        return;
    }

    // Check win condition: no attackers AND no undead left
    if attackers_alive == 0 && undead_alive == 0 {
        *game_outcome = GameOutcome::Victory;
        next_state.set(InGameState::GameOver);
    }
}
