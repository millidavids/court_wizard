use bevy::prelude::*;

use super::super::attack_cycle::GlobalAttackCycle;

/// Advances the global attack cycle timer each game frame.
///
/// This timer cycles from 0.0 to cycle_duration seconds, creating a rotating
/// schedule for unit attacks that is consistent across different frame rates.
pub fn tick_attack_cycle(time: Res<Time>, mut attack_cycle: ResMut<GlobalAttackCycle>) {
    attack_cycle.tick(time.delta_secs());
}

/// Ticks the elapsed game time using real (wall-clock) time.
/// Uses `Time<Real>` so the displayed clock is not affected by the between-wave
/// staging speedup or any other virtual-time scaling.
pub fn tick_elapsed_time(
    time: Res<Time<Real>>,
    mut kill_stats: ResMut<super::super::resources::KillStats>,
) {
    kill_stats.elapsed_time += time.delta_secs();
}

/// Sets the virtual time speed from the GameConfig game_speed setting.
pub(crate) fn apply_game_speed(
    config: Res<crate::config::GameConfig>,
    mut time: ResMut<Time<Virtual>>,
) {
    time.set_relative_speed_f64(config.game_speed as f64);
}

/// Pauses gameplay when the window loses focus.
pub(crate) fn auto_pause_on_focus_loss(
    mut focus_events: MessageReader<bevy::window::WindowFocused>,
    mut next_state: ResMut<NextState<crate::state::InGameState>>,
) {
    for event in focus_events.read() {
        if !event.focused {
            next_state.set(crate::state::InGameState::Paused);
        }
    }
}
