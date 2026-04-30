use bevy::prelude::*;

use crate::config::{GameConfig, WizardType};
use crate::state::InGameState;

use super::resources::{GameOutcome, KillStats, WaveState};
use super::units::boss::lich::components::Lich;
use super::units::components::{Corpse, Team};
use super::units::king::components::King;
use super::units::wizard::archetypes::psychopath::constants::DEFENDER_KILL_THRESHOLD;

/// Checks win/lose conditions every frame and transitions to ScoreScreen state.
///
/// Win: All Attackers AND Undead are dead AND all waves have spawned
/// Lose: All Defenders are dead OR King is dead
/// Psychopath: Must also kill 80% of defenders to win
/// Lich override: On lich levels, killing the Lich + clearing every other
/// enemy resolves to Victory even if the King fell or all defenders died
/// in the final exchange — slaying the boss is the level's win condition.
#[allow(clippy::too_many_arguments)]
pub fn check_win_lose_conditions(
    mut next_state: ResMut<NextState<InGameState>>,
    mut game_outcome: ResMut<GameOutcome>,
    units: Query<&Team, Without<Corpse>>,
    dead_kings: Query<&King, With<Corpse>>,
    dead_lich: Query<(), (With<Lich>, With<Corpse>)>,
    wave_state: Res<WaveState>,
    config: Res<GameConfig>,
    kill_stats: Res<KillStats>,
    lich_pending: Option<Res<super::units::boss::lich::components::LichSpawnPending>>,
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

    // Lich-slain Victory override. If the player killed the Lich and there are
    // no other enemies left, count it as Victory before the king-death and
    // defenders-gone checks fire. This prevents the boss kill from being
    // recorded as a defeat just because the king or last defender fell in the
    // same exchange that killed the Lich.
    let lich_slain = !dead_lich.is_empty();
    if lich_slain && attackers_alive == 0 && undead_alive == 0 && wave_state.waves_complete {
        *game_outcome = GameOutcome::Victory;
        next_state.set(InGameState::ScoreScreen);
        return;
    }

    // Check King death (highest priority lose condition outside the lich override)
    if dead_kings.iter().next().is_some() {
        *game_outcome = GameOutcome::DefeatKingDied;
        next_state.set(InGameState::ScoreScreen);
        return;
    }

    // Check lose condition: no defenders left
    if defenders_alive == 0 {
        *game_outcome = GameOutcome::Defeat;
        next_state.set(InGameState::ScoreScreen);
        return;
    }

    // Don't allow victory while the Lich boss is still pending spawn
    if lich_pending.is_some() {
        return;
    }

    // Check win condition: no attackers AND no undead left AND all waves spawned
    if attackers_alive == 0 && undead_alive == 0 && wave_state.waves_complete {
        // Psychopath must kill at least 80% of defenders to win
        if config.wizard_type == WizardType::Psychopath {
            let total_defenders = (crate::game::constants::INITIAL_DEFENDER_COUNT
                + crate::game::units::archer::constants::INITIAL_ARCHER_DEFENDER_COUNT)
                as f32;
            let kill_ratio = kill_stats.defenders_killed as f32 / total_defenders;
            if kill_ratio < DEFENDER_KILL_THRESHOLD {
                *game_outcome = GameOutcome::DefeatNotEnoughCarnage;
                next_state.set(InGameState::ScoreScreen);
                return;
            }
        }
        *game_outcome = GameOutcome::Victory;
        next_state.set(InGameState::ScoreScreen);
    }
}
