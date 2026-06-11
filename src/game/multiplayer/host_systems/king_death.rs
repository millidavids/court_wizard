use bevy::prelude::*;

use crate::game::resources::GameOutcome;
use crate::game::units::components::{Corpse, Team};
use crate::game::units::king::components::King;
use crate::networking::protocol::{GameOverResult, NetworkMessage};
use crate::networking::resources::NetworkConnection;
use crate::state::MultiplayerGameState;

/// Checks for King death and triggers game-over transition.
///
/// Host = Defenders, Guest = Attackers. When a King becomes a corpse:
/// - Defender King dies → guest wins
/// - Attacker King dies → host wins
pub fn check_mp_king_death(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    mut game_outcome: ResMut<GameOutcome>,
    mut next_state: ResMut<NextState<MultiplayerGameState>>,
    kill_stats: Res<crate::game::resources::KillStats>,
    local_stats: Res<crate::game::multiplayer::score_stats::LocalWizardStats>,
    dead_kings: Query<&Team, (With<King>, With<Corpse>)>,
) {
    if let Some(team) = dead_kings.iter().next() {
        let result = match team {
            Team::Defenders => GameOverResult::GuestWins,
            Team::Attackers | Team::Undead => GameOverResult::HostWins,
        };
        end_mp_match(
            result,
            &mut commands,
            &mut connection,
            &mut game_outcome,
            &mut next_state,
            &kill_stats,
            &local_stats,
        );
    }
}

/// Host-authoritative match-end: sets the outcome, assembles the scoreboard,
/// ships `GameOver` to the guest, and transitions to the score screen. Shared by
/// King-death and forfeit.
pub(crate) fn end_mp_match(
    result: GameOverResult,
    commands: &mut Commands,
    connection: &mut NetworkConnection,
    game_outcome: &mut GameOutcome,
    next_state: &mut NextState<MultiplayerGameState>,
    kill_stats: &crate::game::resources::KillStats,
    local_stats: &crate::game::multiplayer::score_stats::LocalWizardStats,
) {
    *game_outcome = match result {
        GameOverResult::HostWins => GameOutcome::Victory,
        GameOverResult::GuestWins => GameOutcome::DefeatKingDied,
    };

    // Build the host's authoritative side-level summary for both peers.
    let defenders_killed = kill_stats.defenders_killed;
    let attackers_and_undead_killed = kill_stats.attackers_killed + kill_stats.undead_killed;
    let summary = crate::networking::protocol::HostMatchSummary {
        defenders_killed,
        attackers_and_undead_killed,
        host_spell_damage: local_stats.spell_damage,
        host_spell_healing: local_stats.spell_healing,
    };

    // Host commands the Defenders. Insert the host's scoreboard with its own side
    // filled in now; the enemy (guest) wizard's spell stats arrive via
    // `WizardStatsReport` and fill in reactively on the score screen.
    commands.insert_resource(crate::game::multiplayer::score_stats::MatchStats::assemble(
        true,
        defenders_killed,
        attackers_and_undead_killed,
        local_stats.spell_damage,
        local_stats.spell_healing,
        0.0,
        0.0,
    ));

    connection
        .outgoing_messages
        .push(NetworkMessage::GameOver { result, summary });
    next_state.set(MultiplayerGameState::ScoreScreen);
}
