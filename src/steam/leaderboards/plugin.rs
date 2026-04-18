use bevy::prelude::*;
use bevy_steamworks::Client;

use crate::state::{AppState, InGameState};
use crate::ui::accumulate_mode_level_stats;

use super::resources::LeaderboardHandles;
use super::systems::{
    drain_leaderboard_handle_results, handles_incomplete, prewarm_leaderboard_handles,
    submit_run_scores_to_steam,
};

/// Registers Steam leaderboard pre-warming, handle draining, and run-end submission.
///
/// Registered only when Steam initialization succeeds, so all systems here can
/// assume the `Client` resource exists. Systems additionally use
/// `run_if(resource_exists::<Client>)` as a defensive guard.
pub(crate) struct LeaderboardsPlugin;

impl Plugin for LeaderboardsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LeaderboardHandles>()
            .add_systems(
                OnEnter(AppState::MainMenu),
                prewarm_leaderboard_handles.run_if(resource_exists::<Client>),
            )
            .add_systems(
                Update,
                drain_leaderboard_handle_results
                    .run_if(resource_exists::<Client>)
                    .run_if(handles_incomplete),
            )
            .add_systems(
                OnEnter(InGameState::ScoreScreen),
                submit_run_scores_to_steam
                    .after(accumulate_mode_level_stats)
                    .run_if(resource_exists::<Client>),
            );
    }
}
