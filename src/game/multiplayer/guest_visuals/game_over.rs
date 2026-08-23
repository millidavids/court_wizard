use bevy::prelude::*;

use crate::game::resources::GameOutcome;
use crate::networking::protocol::{GameOverResult, NetworkMessage};
use crate::networking::resources::NetworkConnection;
use crate::state::MultiplayerGameState;

/// Listens for `GameOver` messages from the host and transitions to the score screen.
pub fn handle_game_over_message(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    mut game_outcome: ResMut<GameOutcome>,
    mut next_state: ResMut<NextState<MultiplayerGameState>>,
    local_stats: Option<Res<crate::game::multiplayer::score_stats::LocalWizardStats>>,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }

    let messages: Vec<NetworkMessage> = std::mem::take(&mut connection.incoming_messages);
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::GameOver { result, summary } => {
                *game_outcome = match result {
                    GameOverResult::HostWins => GameOutcome::DefeatKingDied,
                    GameOverResult::GuestWins => GameOutcome::Victory,
                };

                // Guest commands the Attackers. Map the host's side-neutral
                // summary into the guest's perspective and fill in our own
                // wizard's spell stats locally.
                let (your_damage, your_healing) = local_stats
                    .as_ref()
                    .map(|s| (s.spell_damage, s.spell_healing))
                    .unwrap_or((0.0, 0.0));
                commands.insert_resource(
                    crate::game::multiplayer::score_stats::MatchStats::assemble(
                        false, // guest commands the Attackers
                        summary.defenders_killed,
                        summary.attackers_and_undead_killed,
                        your_damage,
                        your_healing,
                        summary.host_spell_damage,
                        summary.host_spell_healing,
                    ),
                );

                // Report our wizard's spell stats so the host can fill in its
                // enemy column.
                connection
                    .outgoing_messages
                    .push(NetworkMessage::WizardStatsReport {
                        spell_damage: your_damage,
                        spell_healing: your_healing,
                    });

                next_state.set(MultiplayerGameState::ScoreScreen);
            }
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}
