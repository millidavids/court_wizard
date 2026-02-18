use bevy::prelude::*;

use crate::state::InGameState;

use super::resources::GameOutcome;
use super::units::components::{Corpse, Team};
use super::units::king::components::King;

/// Checks win/lose conditions every frame and transitions to ScoreScreen state.
///
/// Win: All Attackers AND Undead are dead (only Defenders remain)
/// Lose: All Defenders are dead OR King is dead
pub fn check_win_lose_conditions(
    mut next_state: ResMut<NextState<InGameState>>,
    mut game_outcome: ResMut<GameOutcome>,
    units: Query<&Team, Without<Corpse>>,
    dead_kings: Query<&King, With<Corpse>>,
) {
    // Check King death first (highest priority lose condition)
    // If a dead King corpse exists, the game is lost
    if dead_kings.iter().next().is_some() {
        *game_outcome = GameOutcome::DefeatKingDied;
        next_state.set(InGameState::ScoreScreen);
        return;
    }

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
        next_state.set(InGameState::ScoreScreen);
        return;
    }

    // Check win condition: no attackers AND no undead left
    if attackers_alive == 0 && undead_alive == 0 {
        *game_outcome = GameOutcome::Victory;
        next_state.set(InGameState::ScoreScreen);
    }
}
