use bevy::prelude::*;

use crate::state::InGameState;

use super::components::PersistentSpellEffect;
use super::resources::GameOutcome;
use super::units::components::{Corpse, Team};
use super::units::king::components::{King, KingSpawned};

/// Checks win/lose conditions every frame and transitions to GameOver state.
///
/// Win: All Attackers AND Undead are dead (only Defenders remain) AND no persistent spell effects exist
/// Lose: All Defenders are dead OR King is dead (only checked after spell effects expire)
///
/// The game will not end while persistent spell effects (like Black Hole or Wall of Stone) exist.
/// This prevents premature victory/defeat when dangerous spells could still change the outcome.
pub fn check_win_lose_conditions(
    mut next_state: ResMut<NextState<InGameState>>,
    mut game_outcome: ResMut<GameOutcome>,
    units: Query<&Team, Without<Corpse>>,
    king_spawned: Res<KingSpawned>,
    kings: Query<&King, Without<Corpse>>,
    spell_effects: Query<&PersistentSpellEffect>,
) {
    // Don't end the game while persistent spell effects exist
    if !spell_effects.is_empty() {
        return;
    }
    // Check King death first (highest priority lose condition)
    if king_spawned.0 && kings.iter().next().is_none() {
        *game_outcome = GameOutcome::DefeatKingDied;
        next_state.set(InGameState::GameOver);
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
        next_state.set(InGameState::GameOver);
        return;
    }

    // Check win condition: no attackers AND no undead left
    if attackers_alive == 0 && undead_alive == 0 {
        *game_outcome = GameOutcome::Victory;
        next_state.set(InGameState::GameOver);
    }
}
