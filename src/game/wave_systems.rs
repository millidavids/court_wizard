//! Wave spawning system — spawns attacker waves at intervals during gameplay.

use std::collections::HashSet;

use bevy::prelude::*;

use super::constants::*;
use super::loading::constants::{MAX_COMMANDER_ARCHERS, MAX_COMMANDER_INFANTRY};
use super::loading::{upgrade_selection, upgrade_systems};
use super::messages::WaveSpawnedMessage;
use super::pathfinding::StagingAttacker;
use super::resources::{CurrentLevel, KillStats, PendingWaveUpgrades, WaveState};
use super::units::aerialist::resources::AerialistAssets;
use super::units::archer::resources::ArcherAssets;
use super::units::brute::constants::BRUTE_START_TIER;
use super::units::components::{Corpse, Hitbox};
use super::units::dispeller::resources::DispellerAssets;
use super::units::healer::resources::HealerAssets;
use super::units::infantry::resources::InfantryAssets;
use super::units::shielder::resources::ShielderAssets;
use super::units::boss::dark_mage::resources::DarkMageAssets;
use super::units::boss::hags::resources::HagAssets;
use super::units::boss::ogre::resources::OgreAssets;
use super::units::{aerialist, archer, boss, brute, infantry};

/// Ticks the wave timer and spawns the next wave when it expires.
/// Does not tick while staging attackers exist (current wave hasn't activated yet).
#[allow(clippy::too_many_arguments)]
pub fn tick_wave_timer(
    time: Res<Time>,
    mut wave_state: ResMut<WaveState>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    current_level: Res<CurrentLevel>,
    infantry_assets: Res<InfantryAssets>,
    archer_assets: Res<ArcherAssets>,
    aerialist_assets: Res<AerialistAssets>,
    mut kill_stats: ResMut<KillStats>,
    mut wave_events: MessageWriter<WaveSpawnedMessage>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    staging_query: Query<(), (With<StagingAttacker>, Without<Corpse>)>,
    roguelite_modifiers: Option<Res<crate::game::game_mode::components::RogueliteModifiers>>,
    active_toggles: Option<Res<crate::game::game_mode::components::ActiveToggles>>,
    boss_assets: (
        Option<Res<OgreAssets>>,
        Option<Res<HagAssets>>,
        Option<Res<DarkMageAssets>>,
    ),
) {
    let (ogre_assets, hag_assets, dark_mage_assets) = boss_assets;

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
    let base_count_mult = roguelite_modifiers
        .as_ref()
        .map(|m| m.enemy_count)
        .unwrap_or(1.0);

    // Rising Tide: each successive wave spawns 25% more enemies
    let rising_tide_mult = if active_toggles
        .as_ref()
        .is_some_and(|t| t.is_active(crate::game::game_mode::components::ToggleModifier::RisingTide))
    {
        1.0 + 0.25 * next_wave as f32
    } else {
        1.0
    };
    let count_mult = base_count_mult * rising_tide_mult;

    // Spawn infantry for this wave, collecting entity IDs
    let total_infantry = (calculate_total_infantry(level) as f32 * count_mult).round() as u32;
    let mut infantry_entities = Vec::with_capacity(total_infantry as usize);
    for i in 0..total_infantry {
        let entity = infantry::systems::spawn_single_attacker(
            &mut game_rng.0,
            &mut commands,
            &infantry_assets,
            &mut materials,
            i,
            level,
        );
        infantry_entities.push(entity);
    }

    // Spawn archers for this wave, collecting entity IDs
    let total_archers = (calculate_total_archers(level) as f32 * count_mult).round() as u32;
    let mut archer_entities = Vec::with_capacity(total_archers as usize);
    for i in 0..total_archers {
        let entity = archer::systems::spawn_single_attacker_archer(
            &mut game_rng.0,
            &mut commands,
            &archer_assets,
            &mut materials,
            i,
            level,
        );
        archer_entities.push(entity);
    }

    // Spawn aerialists for this wave (tier 2+), collecting entity IDs
    let total_aerialists = (calculate_total_aerialists(level) as f32 * count_mult).round() as u32;
    let mut aerialist_entities = Vec::with_capacity(total_aerialists as usize);
    for i in 0..total_aerialists {
        let entity = aerialist::systems::spawn_single_attacker_aerialist(
            &mut game_rng.0,
            &mut commands,
            &aerialist_assets,
            &mut materials,
            i,
            level,
        );
        aerialist_entities.push(entity);
    }

    // Spawn brute if tier qualifies
    let has_brute = get_tier(level) >= BRUTE_START_TIER;
    if has_brute {
        brute::systems::spawn_brute(
            &mut game_rng.0,
            commands.reborrow(),
            Res::clone(&infantry_assets),
            &mut materials,
            Res::clone(&current_level),
        );
    }

    // Boss Parade: spawn a boss every 3rd wave (cycling Hag → Ogre → Dark Mage)
    let boss_parade_spawned = if active_toggles.as_ref().is_some_and(|t| {
        t.is_active(crate::game::game_mode::components::ToggleModifier::BossParade)
    }) && next_wave % 3 == 2
    {
        match (next_wave / 3) % 3 {
            0 => {
                if let Some(ref assets) = hag_assets {
                    boss::hags::systems::spawn_hags(&mut game_rng.0, commands.reborrow(), Res::clone(assets));
                    3 // hags spawn 3 units
                } else {
                    0
                }
            }
            1 => {
                if let Some(ref assets) = ogre_assets {
                    boss::ogre::systems::spawn_ogre(
                        &mut game_rng.0,
                        commands.reborrow(),
                        Res::clone(assets),
                        &mut materials,
                    );
                    1
                } else {
                    0
                }
            }
            _ => {
                if let Some(ref assets) = dark_mage_assets {
                    boss::dark_mage::systems::spawn_dark_mage(
                        &mut game_rng.0,
                        commands.reborrow(),
                        Res::clone(assets),
                    );
                    1
                } else {
                    0
                }
            }
        }
    } else {
        0
    };

    // Update kill stats with newly spawned attackers
    let wave_attackers = total_infantry
        + total_archers
        + total_aerialists
        + if has_brute { 1 } else { 0 }
        + boss_parade_spawned;
    kill_stats.total_attackers_spawned += wave_attackers;

    // Check if this was the last wave
    if next_wave + 1 >= wave_state.total_waves {
        wave_state.waves_complete = true;
    }

    // Store spawned entities for deferred upgrade processing (next frame)
    commands.insert_resource(PendingWaveUpgrades {
        wave: next_wave,
        infantry: infantry_entities,
        archers: archer_entities,
        aerialists: aerialist_entities,
    });

    // Notify UI
    wave_events.write(WaveSpawnedMessage {
        wave_number: next_wave + 1, // 1-indexed for display
    });
}

/// Applies elite, commander, dispeller, healer, and shielder upgrades to
/// units from a newly spawned wave. Runs on the frame after wave spawning
/// so that deferred commands have flushed and entities are queryable.
#[allow(clippy::too_many_arguments)]
pub fn apply_wave_upgrades(
    mut commands: Commands,
    pending: Res<PendingWaveUpgrades>,
    current_level: Res<CurrentLevel>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    dispeller_assets: Res<DispellerAssets>,
    healer_assets: Res<HealerAssets>,
    shielder_assets: Res<ShielderAssets>,
    spell_visual_assets: Res<crate::game::units::wizard::spells::visual_assets::SpellVisualAssets>,
    transform_query: Query<&Transform>,
    hitbox_query: Query<&Hitbox>,
) {
    let level = current_level.0;
    let wave = pending.wave;
    // Use wave offset so each wave gets different RNG sequences
    let seed_base = level as u64 + wave as u64 * 10000;

    let mut excluded = HashSet::new();

    let infantry_commanders = upgrade_selection::select_commander_entities(
        &pending.infantry,
        level,
        MAX_COMMANDER_INFANTRY,
        seed_base,
        1,
        "Infantry (wave)",
    );
    excluded.extend(&infantry_commanders);

    let archer_commanders = upgrade_selection::select_commander_entities(
        &pending.archers,
        level,
        MAX_COMMANDER_ARCHERS,
        seed_base,
        997,
        "Archer (wave)",
    );
    excluded.extend(&archer_commanders);

    let dispellers =
        upgrade_selection::select_dispeller_entities(&pending.archers, level, &excluded, seed_base);
    excluded.extend(&dispellers);

    let healers =
        upgrade_selection::select_healer_entities(&pending.archers, level, &excluded, seed_base);
    excluded.extend(&healers);

    let shielders =
        upgrade_selection::select_shielder_entities(&pending.infantry, level, &excluded, seed_base);
    excluded.extend(&shielders);

    // Elites run last so all other upgrade types are excluded first
    let mut all_wave_entities = Vec::with_capacity(
        pending.infantry.len() + pending.archers.len() + pending.aerialists.len(),
    );
    all_wave_entities.extend(&pending.infantry);
    all_wave_entities.extend(&pending.archers);
    all_wave_entities.extend(&pending.aerialists);

    let elites =
        upgrade_selection::select_elite_entities(&all_wave_entities, level, &excluded, seed_base);

    for entity in infantry_commanders.iter().chain(archer_commanders.iter()) {
        if let Some((transform, hitbox)) =
            upgrade_systems::get_transform_and_hitbox(*entity, &transform_query, &hitbox_query)
        {
            upgrade_systems::apply_commander_upgrade(
                &mut commands,
                *entity,
                &mut materials,
                &mut meshes,
                &spell_visual_assets,
                &transform,
                &hitbox,
            );
        }
    }

    for entity in &dispellers {
        upgrade_systems::apply_dispeller_upgrade(
            &mut commands,
            *entity,
            &dispeller_assets,
            &mut materials,
        );
    }

    for entity in &healers {
        upgrade_systems::apply_healer_upgrade(
            &mut commands,
            *entity,
            &healer_assets,
            &mut materials,
        );
    }

    for entity in &shielders {
        upgrade_systems::apply_shielder_upgrade(
            &mut commands,
            *entity,
            &shielder_assets,
            &mut materials,
        );
    }

    for entity in &elites {
        if let Some((transform, hitbox)) =
            upgrade_systems::get_transform_and_hitbox(*entity, &transform_query, &hitbox_query)
        {
            upgrade_systems::apply_elite_upgrade(&mut commands, *entity, &transform, &hitbox);
        }
    }

    commands.remove_resource::<PendingWaveUpgrades>();
}
