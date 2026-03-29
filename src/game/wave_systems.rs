//! Wave spawning system — spawns attacker waves at intervals during gameplay.

use bevy::prelude::*;

use super::constants::*;
use super::messages::WaveSpawnedMessage;
use super::pathfinding::StagingAttacker;
use super::resources::{CurrentLevel, KillStats, WaveState};
use super::units::components::Corpse;
use super::units::aerialist::resources::AerialistAssets;
use super::units::archer::resources::ArcherAssets;
use super::units::brute::constants::BRUTE_START_TIER;
use super::units::infantry::resources::InfantryAssets;
use super::units::{aerialist, archer, brute, infantry};

/// Ticks the wave timer and spawns the next wave when it expires.
/// Does not tick while staging attackers exist (current wave hasn't activated yet).
#[allow(clippy::too_many_arguments)]
pub fn tick_wave_timer(
    time: Res<Time>,
    mut wave_state: ResMut<WaveState>,
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    infantry_assets: Res<InfantryAssets>,
    archer_assets: Res<ArcherAssets>,
    aerialist_assets: Res<AerialistAssets>,
    mut kill_stats: ResMut<KillStats>,
    mut wave_events: MessageWriter<WaveSpawnedMessage>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    staging_query: Query<(), (With<StagingAttacker>, Without<Corpse>)>,
    roguelite_modifiers: Option<Res<crate::game::game_mode::components::RogueliteModifiers>>,
) {
    if wave_state.waves_complete {
        return;
    }

    // Don't tick wave timer while living staging attackers exist — wait for current wave to activate
    if !staging_query.is_empty() {
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
    let count_mult = roguelite_modifiers
        .as_ref()
        .map(|m| m.enemy_count)
        .unwrap_or(1.0);

    // Spawn infantry for this wave
    let total_infantry = (calculate_total_infantry(level) as f32 * count_mult).round() as u32;
    for i in 0..total_infantry {
        infantry::systems::spawn_single_attacker(
            &mut commands,
            &infantry_assets,
            &mut materials,
            i,
            level,
        );
    }

    // Spawn archers for this wave
    let total_archers = (calculate_total_archers(level) as f32 * count_mult).round() as u32;
    for i in 0..total_archers {
        archer::systems::spawn_single_attacker_archer(
            &mut commands,
            &archer_assets,
            &mut materials,
            i,
            level,
        );
    }

    // Spawn aerialists for this wave (tier 2+)
    let total_aerialists = (calculate_total_aerialists(level) as f32 * count_mult).round() as u32;
    for i in 0..total_aerialists {
        aerialist::systems::spawn_single_attacker_aerialist(
            &mut commands,
            &aerialist_assets,
            &mut materials,
            i,
            level,
        );
    }

    // Spawn brute if tier qualifies
    let has_brute = get_tier(level) >= BRUTE_START_TIER;
    if has_brute {
        brute::systems::spawn_brute(
            commands.reborrow(),
            Res::clone(&infantry_assets),
            &mut materials,
            Res::clone(&current_level),
        );
    }

    // Update kill stats with newly spawned attackers
    let wave_attackers = total_infantry + total_archers + total_aerialists + if has_brute { 1 } else { 0 };
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
