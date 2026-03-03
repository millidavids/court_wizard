//! Wave spawning system — spawns attacker waves at intervals during gameplay.

use bevy::prelude::*;

use super::constants::*;
use super::messages::WaveSpawnedMessage;
use super::resources::{CurrentLevel, KillStats, WaveState};
use super::units::archer::resources::ArcherAssets;
use super::units::brute::constants::BRUTE_START_TIER;
use super::units::brute::resources::BruteAssets;
use super::units::infantry::resources::InfantryAssets;
use super::units::{archer, brute, infantry};

/// Ticks the wave timer and spawns the next wave when it expires.
#[allow(clippy::too_many_arguments)]
pub fn tick_wave_timer(
    time: Res<Time>,
    mut wave_state: ResMut<WaveState>,
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    infantry_assets: Res<InfantryAssets>,
    archer_assets: Res<ArcherAssets>,
    brute_assets: Res<BruteAssets>,
    mut kill_stats: ResMut<KillStats>,
    mut wave_events: MessageWriter<WaveSpawnedMessage>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if wave_state.waves_complete {
        return;
    }

    wave_state.wave_timer -= time.delta_secs();

    if wave_state.wave_timer > 0.0 {
        return;
    }

    // Timer expired — spawn the next wave
    let next_wave = wave_state.current_wave + 1;
    if next_wave >= wave_state.total_waves {
        // All waves already spawned
        wave_state.waves_complete = true;
        return;
    }

    wave_state.current_wave = next_wave;
    wave_state.wave_timer = wave_state.wave_interval;

    let level = current_level.0;

    // Spawn infantry for this wave
    let total_infantry = calculate_total_infantry(level);
    for i in 0..total_infantry {
        infantry::systems::spawn_single_attacker(&mut commands, &infantry_assets, &mut materials, i, level);
    }

    // Spawn archers for this wave
    let total_archers = calculate_total_archers(level);
    for i in 0..total_archers {
        archer::systems::spawn_single_attacker_archer(&mut commands, &archer_assets, &mut materials, i, level);
    }

    // Spawn brute if tier qualifies
    let has_brute = get_tier(level) >= BRUTE_START_TIER;
    if has_brute {
        brute::systems::spawn_brute(
            commands.reborrow(),
            Res::clone(&brute_assets),
            Res::clone(&current_level),
        );
    }

    // Update kill stats with newly spawned attackers
    let wave_attackers = total_infantry + total_archers + if has_brute { 1 } else { 0 };
    kill_stats.total_attackers_spawned += wave_attackers;

    // Check if this was the last wave
    if next_wave + 1 >= wave_state.total_waves {
        wave_state.waves_complete = true;
    }

    // Notify UI
    wave_events.write(WaveSpawnedMessage {
        wave_number: next_wave + 1, // 1-indexed for display
    });
}
