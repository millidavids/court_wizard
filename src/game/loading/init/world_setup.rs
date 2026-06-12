use bevy::prelude::*;

use crate::config::GameConfig;
use crate::game::constants::*;
use crate::game::loading::spawn_queue::{SpawnQueue, SpawnTask};
use crate::game::resources::{
    CurrentLevel, InitialDefenderCount, KillStats, TimeTravelState, WaveState,
};
use crate::game::units::archer::constants::INITIAL_ARCHER_DEFENDER_COUNT;

/// Initializes the loading progress tracker and spawn queue.
#[allow(clippy::too_many_arguments)]
pub fn init_loading_progress(
    mut commands: Commands,
    mut current_level: ResMut<CurrentLevel>,
    mut kill_stats: ResMut<KillStats>,
    mut config: ResMut<GameConfig>,
    time_travel: Option<Res<TimeTravelState>>,
    active_talents: Option<Res<crate::game::units::wizard::talents::resources::ActiveTalents>>,
    game_mode: Option<Res<crate::game::game_mode::components::GameMode>>,
    roguelite_modifiers: Option<Res<crate::game::game_mode::components::RogueliteModifiers>>,
    active_toggles: Option<Res<crate::game::game_mode::components::ActiveToggles>>,
    attrition_state: Option<Res<crate::game::game_mode::components::AttritionState>>,
    coop_pending: Option<Res<crate::game::multiplayer::coop::CoopPendingSession>>,
) {
    // Sync CurrentLevel from GameConfig, but skip during time travel
    // (CurrentLevel was already overridden by the wizard tower hub)
    if time_travel.is_none() {
        current_level.0 = config.current_level;
    }

    // Why: not every entry path (e.g. roguelite Continue Run, victory → tower → next level)
    // resets kill_stats, and any stale values get re-accumulated into lifetime totals
    // by send_battle_ended.
    kill_stats.reset();

    // Co-op host: the wizard-tower start flow left a `CoopPendingSession` when
    // launching this endless/roguelite level with a guest connected. Promote it
    // to a real host `MultiplayerSession` (before the battlefield is built) so the
    // networking layer treats this InGame match as co-op, enqueue the guest's
    // wizard proxy, and stand up the load handshake. Each endless/roguelite level
    // loops through the tower, so this runs fresh per co-op level.
    let coop_urgent = active_toggles
        .as_ref()
        .is_some_and(|t| t.is_active(crate::game::game_mode::components::ToggleModifier::Urgent));
    let coop_guest_wizard = coop_pending.as_deref().map(|pending| {
        commands.insert_resource(crate::networking::session::MultiplayerSession {
            role: crate::networking::resources::PeerRole::Host,
            mode: pending.mode,
            host_wizard: config.wizard_type,
            guest_wizard: pending.guest_wizard,
            host_spells: Vec::new(),
            guest_spells: pending.guest_spells.clone(),
            // Co-op roguelite Urgent disables pause-sync (game keeps running).
            coop_urgent: coop_urgent
                && pending.mode == crate::networking::session::SessionMode::CoopRoguelite,
        });
        pending.guest_wizard
    });
    if coop_guest_wizard.is_some() {
        commands.remove_resource::<crate::game::multiplayer::coop::CoopPendingSession>();
        // Stand up the "both peers loaded" handshake so the host (loading on this
        // single-player path) waits for the guest before entering the match.
        commands.insert_resource(crate::game::multiplayer::coop::CoopLoadingSync::default());
    }

    let mut queue = SpawnQueue::new();
    let level = current_level.0;

    // Spawn in intelligent order: Battlefield -> Castle -> Grid -> King -> Infantry -> Archers -> Brute/Ogre -> Wizard

    // 1. Battlefield (foundation)
    queue.tasks.push_back(SpawnTask::Battlefield);

    // 1b. Trampling overlay (mud effect on top of ground tiles)
    queue.tasks.push_back(SpawnTask::TramplingOverlay);

    // 2. Castle (part of battlefield)
    queue.tasks.push_back(SpawnTask::Castle);

    // 3. Pathfinding Grid (needed for unit movement)
    queue.tasks.push_back(SpawnTask::PathfindingGrid);

    // 3b. Permanent walls from previous victories (after pathfinding grid)
    for saved_wall in &config.saved_walls {
        queue.tasks.push_back(SpawnTask::PermanentWall {
            wall: saved_wall.clone(),
        });
    }

    // 3c. Permanent crystals from previous victories (after pathfinding grid)
    if !config.saved_crystals.is_empty() {
        let crystal_talent_params =
            crate::game::units::wizard::spells::arcane_crystal::systems::compute_talent_params(
                active_talents.as_deref(),
            );
        for saved_crystal in &config.saved_crystals {
            queue.tasks.push_back(SpawnTask::PermanentCrystal {
                crystal: saved_crystal.clone(),
                damage_mult: crystal_talent_params.damage_mult,
                count_mult: crystal_talent_params.count_mult,
                resonance_cascade: crystal_talent_params.resonance_cascade,
            });
        }
    }

    // 3d. Battlefield flora (generate on first battle, then spawn from save)
    if config.saved_flora.is_empty() {
        crate::game::terrain::flora::systems::generate_flora_positions(&mut config);
    }
    for flora in &config.saved_flora {
        queue.tasks.push_back(SpawnTask::Flora {
            flora: flora.clone(),
        });
    }

    // 3e. Terrain (trees, ponds, bushes, boulders) — generate on first battle, spawn from save
    {
        let terrain_density = roguelite_modifiers
            .as_ref()
            .map(|m| m.terrain_density)
            .unwrap_or(1.0);

        let has_no_terrain = config.saved_trees.is_empty()
            && config.saved_ponds.is_empty()
            && config.saved_bushes.is_empty()
            && config.saved_boulders.is_empty();

        if has_no_terrain {
            crate::game::loading::terrain_generation::generate_terrain(
                &mut config,
                level,
                terrain_density,
                false, // single-player: no mirrored forest
            );
        }

        for boulder in &config.saved_boulders {
            queue.tasks.push_back(SpawnTask::TerrainBoulder {
                boulder: boulder.clone(),
            });
        }
        for tree in &config.saved_trees {
            queue
                .tasks
                .push_back(SpawnTask::TerrainTree { tree: tree.clone() });
        }
        for pond in &config.saved_ponds {
            queue
                .tasks
                .push_back(SpawnTask::TerrainPond { pond: pond.clone() });
        }
        for bush in &config.saved_bushes {
            queue
                .tasks
                .push_back(SpawnTask::TerrainBush { bush: bush.clone() });
        }
    }

    // 4. King (central defender)
    queue.tasks.push_back(SpawnTask::King);

    // Veteran Defenders toggle: halve defender counts
    let veteran_defenders = active_toggles.as_ref().is_some_and(|t| {
        t.is_active(crate::game::game_mode::components::ToggleModifier::VeteranDefenders)
    });
    let defender_mult = if veteran_defenders { 0.5 } else { 1.0 };

    // Attrition toggle: use surviving counts from previous level if available
    let base_guard_count = attrition_state
        .as_ref()
        .map(|a| a.guards)
        .unwrap_or(KINGS_GUARD_COUNT);
    let base_infantry_count = attrition_state
        .as_ref()
        .map(|a| a.infantry)
        .unwrap_or(INITIAL_DEFENDER_COUNT);

    // 5. King's Guard (protect the king)
    let guard_count = (base_guard_count as f32 * defender_mult).round() as u32;
    for i in 0..guard_count {
        queue
            .tasks
            .push_back(SpawnTask::KingsGuard { guard_index: i });
    }

    // 6. Defender Infantry
    let infantry_count = (base_infantry_count as f32 * defender_mult).round() as u32;
    for i in 0..infantry_count {
        queue
            .tasks
            .push_back(SpawnTask::DefenderInfantry { unit_index: i });
    }

    // Check if this is a boss level (every 5th level starting at 5)
    use crate::game::units::brute::constants::BRUTE_START_TIER;
    use crate::game::units::teleporter::constants::TELEPORTER_START_TIER;

    if is_boss_level(level) && !is_lich_level(level) {
        let tier = get_tier(level);
        match tier % BOSS_CYCLE_LENGTH {
            0 => {
                queue.tasks.push_back(SpawnTask::Ogre);
                kill_stats.total_attackers_spawned = 1;
            }
            2 => {
                queue.tasks.push_back(SpawnTask::DarkMage);
                kill_stats.total_attackers_spawned = 1;
            }
            3 => {
                queue.tasks.push_back(SpawnTask::Hags);
                kill_stats.total_attackers_spawned = 3;
            }
            4 => {
                queue.tasks.push_back(SpawnTask::Ray);
                kill_stats.total_attackers_spawned = 1;
            }
            _ => {
                queue.tasks.push_back(SpawnTask::Ogre);
                kill_stats.total_attackers_spawned = 1;
            }
        }
        commands.insert_resource(WaveState {
            current_wave: 0,
            total_waves: 1,
            wave_timer: 0.0,
            wave_interval: 0.0,
            waves_complete: true,
        });
    } else {
        // Normal level: spawn wave 1 infantry, archers, and possibly brute
        let wave_count = calculate_wave_count(level);

        // Enemy count multiplier from roguelite modifiers (default 1.0)
        let count_mult = roguelite_modifiers
            .as_ref()
            .map(|m| m.enemy_count)
            .unwrap_or(1.0);

        // 7. Attacker Infantry (wave 1)
        let is_endless = crate::game::game_mode::components::is_endless_mode(game_mode.as_deref());
        let extra_infantry = if is_endless {
            crate::game::constants::endless_extra_infantry(level)
        } else {
            0
        };
        let extra_archers = if is_endless {
            crate::game::constants::endless_extra_archers(level)
        } else {
            0
        };
        let total_attackers =
            ((calculate_total_infantry(level) + extra_infantry) as f32 * count_mult).round() as u32;
        for i in 0..total_attackers {
            queue.tasks.push_back(SpawnTask::AttackerInfantry {
                unit_index: i,
                level,
            });
        }

        // 8. Attacker Archers (wave 1)
        let total_attacker_archers =
            ((calculate_total_archers(level) + extra_archers) as f32 * count_mult).round() as u32;
        for i in 0..total_attacker_archers {
            queue.tasks.push_back(SpawnTask::AttackerArcher {
                unit_index: i,
                level,
            });
        }

        // 8b. Attacker Assassins (wave 1) — start spawning at tier 2
        let total_assassins = (calculate_total_assassins(level) as f32 * count_mult).round() as u32;
        for i in 0..total_assassins {
            queue.tasks.push_back(SpawnTask::AttackerAssassin {
                unit_index: i,
                level,
            });
        }

        // 8c. Attacker Aerialists (wave 1) — start spawning at tier 2
        let total_aerialists =
            (calculate_total_aerialists(level) as f32 * count_mult).round() as u32;
        for i in 0..total_aerialists {
            queue.tasks.push_back(SpawnTask::AttackerAerialist {
                unit_index: i,
                level,
            });
        }

        // 9. Brute (if tier qualifies, wave 1)
        let has_brute = get_tier(level) >= BRUTE_START_TIER;
        if has_brute {
            queue.tasks.push_back(SpawnTask::Brute);
        }

        // 9b. Teleporter (rare infiltrator, 1 per level — gated by tier)
        let has_teleporter = get_tier(level) >= TELEPORTER_START_TIER;
        if has_teleporter {
            queue.tasks.push_back(SpawnTask::Teleporter);
        }

        // Record total attackers spawned across ALL waves for score screen
        let per_wave = total_attackers
            + total_attacker_archers
            + total_assassins
            + total_aerialists
            + if has_brute { 1 } else { 0 }
            + if has_teleporter { 1 } else { 0 };
        kill_stats.total_attackers_spawned = per_wave * wave_count;

        // Initialize wave state (game_speed modifier scales wave interval)
        let speed_mult = roguelite_modifiers
            .as_ref()
            .map(|m| m.game_speed)
            .unwrap_or(1.0);
        let wave_interval = WAVE_INTERVAL_SECONDS / speed_mult;
        commands.insert_resource(WaveState {
            current_wave: 0,
            total_waves: wave_count,
            wave_timer: wave_interval,
            wave_interval,
            waves_complete: wave_count <= 1,
        });

        // Lich levels: schedule the Lich to spawn after wave 3
        if is_lich_level(level) {
            commands.insert_resource(crate::game::units::boss::lich::components::LichSpawnPending);
        }
    }

    // Set retreat count for this level: tier + 1
    {
        let tier = get_tier(level);
        let retreats = tier + 1;
        commands.insert_resource(crate::game::units::infantry::components::RetreatState {
            retreat_timer: 0.0,
            retreats_remaining: retreats,
        });
    }

    // 10. Defender Archers (always spawn regardless of boss level)
    let base_archer_count = attrition_state
        .as_ref()
        .map(|a| a.archers)
        .unwrap_or(INITIAL_ARCHER_DEFENDER_COUNT);
    let archer_def_count = (base_archer_count as f32 * defender_mult).round() as u32;
    for i in 0..archer_def_count {
        queue
            .tasks
            .push_back(SpawnTask::DefenderArcher { unit_index: i });
    }

    // Track initial defender count for spell shield threshold (multiplayer)
    commands.insert_resource(InitialDefenderCount(
        infantry_count + guard_count + archer_def_count,
    ));

    // 11. Load wizard assets (sprite sheet texture)
    queue.tasks.push_back(SpawnTask::LoadWizardAssets);

    // 12. Wizard (controls spells)
    queue.tasks.push_back(SpawnTask::Wizard);

    // 12b. Co-op: the guest's wizard proxy, beside the host on the shared wall.
    if let Some(guest_wizard) = coop_guest_wizard {
        queue
            .tasks
            .push_back(SpawnTask::CoopGuestWizard { guest_wizard });
    }

    // 13. Load cauldron assets (texture for sprite sheet)
    queue.tasks.push_back(SpawnTask::LoadCauldronAssets);

    // 13b. Cauldron (next to wizard on castle wall)
    queue.tasks.push_back(SpawnTask::Cauldron);

    // 14. Select upgrades for attacker units (after all attackers spawn)
    queue.tasks.push_back(SpawnTask::SelectInfantryUpgrades);
    queue.tasks.push_back(SpawnTask::SelectArcherUpgrades);
    queue.tasks.push_back(SpawnTask::SelectDispellerUpgrades);
    queue.tasks.push_back(SpawnTask::SelectHealerUpgrades);
    queue.tasks.push_back(SpawnTask::SelectShielderUpgrades);
    // Elite pass runs LAST — any surviving attacker unit type can become elite
    queue.tasks.push_back(SpawnTask::SelectEliteUpgrades);

    commands.insert_resource(queue);
}
