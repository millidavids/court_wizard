use bevy::prelude::*;

use super::super::cauldron::resources::CauldronBuffs;
use super::super::resources::CurrentLevel;
use super::super::units::king::components::KingSpawned;

/// Initializes the current level from saved config.
///
/// This system runs on OnEnter(AppState::InGame) to restore the player's
/// current level from their last session.
pub fn init_level_from_config(
    mut current_level: ResMut<CurrentLevel>,
    config: Res<crate::config::GameConfig>,
    time_travel: Option<Res<crate::game::resources::TimeTravelState>>,
) {
    // During time travel, CurrentLevel was already set to the target level
    // by the time travel handler. Don't overwrite it with config.current_level
    // which tracks actual progression.
    if time_travel.is_some() {
        return;
    }
    current_level.0 = config.current_level;
}

/// Stops all in-game sound effects by despawning audio entities.
/// Runs when transitioning to the score screen so spell sounds don't persist.
pub fn stop_all_sfx(
    mut commands: Commands,
    audio_query: Query<
        Entity,
        (
            With<AudioPlayer>,
            With<super::super::components::OnGameplayScreen>,
        ),
    >,
) {
    for entity in &audio_query {
        commands.entity(entity).try_despawn();
    }
}

/// Cleans up all game entities when exiting the InGame state.
pub fn cleanup_game(
    mut commands: Commands,
    query: Query<Entity, With<super::super::components::OnGameplayScreen>>,
) {
    // Don't reset level - it persists between sessions via config
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
}

/// Cleans up game entities when replaying (transitioning from ScoreScreen).
///
/// This system runs on OnExit(InGameState::ScoreScreen) and despawns all game entities
/// in preparation for re-spawning them fresh.
pub fn cleanup_for_replay(
    mut commands: Commands,
    gameplay_entities: Query<Entity, With<super::super::components::OnGameplayScreen>>,
) {
    for entity in &gameplay_entities {
        commands.entity(entity).try_despawn();
    }
}

/// Resets game resources when replaying (transitioning from ScoreScreen).
///
/// This system runs on OnExit(InGameState::ScoreScreen) and resets resources like
/// the attack cycle timer and defender activation status.
pub fn reset_resources_for_replay(
    mut attack_cycle: ResMut<super::super::attack_cycle::GlobalAttackCycle>,
    mut defenders_activated: ResMut<super::super::units::infantry::components::DefendersActivated>,
    mut king_spawned: ResMut<KingSpawned>,
    mut cauldron_buffs: ResMut<CauldronBuffs>,
    mut battle_insight: ResMut<super::super::resources::BattleInsightData>,
    mut stone_used: ResMut<super::super::cauldron::resources::PhilosophersStoneUsed>,
    mut retreat_state: ResMut<super::super::units::infantry::components::RetreatState>,
) {
    attack_cycle.current_time = 0.0;
    defenders_activated.active = false;
    king_spawned.0 = false;
    cauldron_buffs.reset();
    stone_used.0 = false;
    *battle_insight = Default::default();
    *retreat_state = Default::default();
    // WaveState is NOT reset here — it's freshly set by init_loading_progress
    // during each level load, so resetting here would overwrite the correct values.
}
