use std::collections::HashSet;

use bevy::prelude::*;

use super::constants::{MAX_COMMANDER_ARCHERS, MAX_COMMANDER_INFANTRY};
use super::spawn_queue::{SpawnQueue, SpawnTask};
use super::upgrade_selection;
use super::upgrade_systems;
use crate::config::GameConfig;
use crate::game::battlefield::components::BattlefieldAssets;
use crate::game::constants::*;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::resources::{
    CurrentLevel, InitialDefenderCount, KillStats, TimeTravelState, WaveState,
};
use crate::game::units::aerialist::AerialistAssets;
use crate::game::units::aerialist::systems as aerialist_systems;
use crate::game::units::archer::constants::INITIAL_ARCHER_DEFENDER_COUNT;
use crate::game::units::archer::systems as archer_systems;
use crate::game::units::archer::{Archer, ArcherAssets};
use crate::game::units::assassin::AssassinAssets;
use crate::game::units::assassin::systems as assassin_systems;
use crate::game::units::components::{Hitbox, Team};
use crate::game::units::infantry::Infantry;
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::infantry::systems as infantry_systems;
use crate::state::AppState;

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
) {
    // Sync CurrentLevel from GameConfig, but skip during time travel
    // (CurrentLevel was already overridden by the wizard tower hub)
    if time_travel.is_none() {
        current_level.0 = config.current_level;
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

    // 5. King's Guard (protect the king)
    for i in 0..KINGS_GUARD_COUNT {
        queue
            .tasks
            .push_back(SpawnTask::KingsGuard { guard_index: i });
    }

    // 6. Defender Infantry
    for i in 0..INITIAL_DEFENDER_COUNT {
        queue
            .tasks
            .push_back(SpawnTask::DefenderInfantry { unit_index: i });
    }

    // Check if this is a boss level (every 5th level starting at 5)
    use crate::game::constants::{BOSS_CYCLE_LENGTH, get_tier};
    use crate::game::constants::{is_boss_level, is_lich_level};
    use crate::game::units::brute::constants::BRUTE_START_TIER;

    if is_boss_level(level) && !is_lich_level(level) {
        let tier = get_tier(level);
        match tier % BOSS_CYCLE_LENGTH {
            0 => {
                queue.tasks.push_back(SpawnTask::Ogre);
                kill_stats.total_attackers_spawned = 1;
            }
            2 => {
                queue.tasks.push_back(SpawnTask::Hags);
                kill_stats.total_attackers_spawned = 3;
            }
            3 => {
                queue.tasks.push_back(SpawnTask::DarkMage);
                kill_stats.total_attackers_spawned = 1;
            }
            _ => {
                // Fallback: Ogre
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

        // Record total attackers spawned across ALL waves for score screen
        let per_wave = total_attackers
            + total_attacker_archers
            + total_assassins
            + total_aerialists
            + if has_brute { 1 } else { 0 };
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

    // 8. Defender Archers (always spawn regardless of boss level)
    for i in 0..INITIAL_ARCHER_DEFENDER_COUNT {
        queue
            .tasks
            .push_back(SpawnTask::DefenderArcher { unit_index: i });
    }

    // Track initial defender count for spell shield threshold (multiplayer)
    commands.insert_resource(InitialDefenderCount(
        INITIAL_DEFENDER_COUNT + KINGS_GUARD_COUNT + INITIAL_ARCHER_DEFENDER_COUNT,
    ));

    // 11. Load wizard assets (sprite sheet texture)
    queue.tasks.push_back(SpawnTask::LoadWizardAssets);

    // 12. Wizard (controls spells)
    queue.tasks.push_back(SpawnTask::Wizard);

    // 13. Load cauldron assets (texture for sprite sheet)
    queue.tasks.push_back(SpawnTask::LoadCauldronAssets);

    // 13. Cauldron (next to wizard on castle wall)
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

/// Processes spawn tasks from the queue, spreading them evenly across frames.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn process_spawn_queue(
    mut commands: Commands,
    mut spawn_queue: ResMut<SpawnQueue>,
    mut next_state: ResMut<NextState<AppState>>,
    // Resources needed for spawning
    config: Res<GameConfig>,
    unit_assets: (
        Res<InfantryAssets>,
        Res<ArcherAssets>,
        Res<AssassinAssets>,
        Res<crate::game::units::dispeller::resources::DispellerAssets>,
        Res<crate::game::units::shielder::resources::ShielderAssets>,
        Res<crate::game::units::healer::resources::HealerAssets>,
    ),
    aerialist_assets: Res<AerialistAssets>,
    king_assets: Res<crate::game::units::king::resources::KingAssets>,
    boss_assets: (
        Res<crate::game::units::boss::ogre::resources::OgreAssets>,
        Res<crate::game::units::boss::hags::resources::HagAssets>,
        Res<crate::game::units::boss::dark_mage::resources::DarkMageAssets>,
    ),
    current_level: Res<CurrentLevel>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut king_spawned: ResMut<crate::game::units::king::components::KingSpawned>,
    optional_assets: (
        Option<Res<crate::game::units::wizard::components::WizardAssets>>,
        Option<Res<crate::game::cauldron::resources::CauldronAssets>>,
        ResMut<Assets<Image>>,
    ),
    shared_assets: (
        Res<BattlefieldAssets>,
        Res<crate::game::units::wizard::spells::visual_assets::SpellVisualAssets>,
        Res<AssetServer>,
        Res<crate::game::terrain::flora::resources::FloraAssets>,
        Res<crate::game::battlefield::trampling::resources::TramplingGrid>,
        Res<crate::game::terrain::tree::resources::TreeAssets>,
        Res<crate::game::terrain::pond::resources::PondAssets>,
        Res<crate::game::terrain::bush::resources::BushAssets>,
        Res<crate::game::terrain::boulder::resources::BoulderAssets>,
        Res<crate::game::shared_systems::ShadowAssets>,
    ),
    // Use ParamSet to reduce parameter count and avoid query conflicts
    mut queries: ParamSet<(
        Query<(Entity, &Team), With<Infantry>>,
        Query<(Entity, &Team), With<Archer>>,
        Query<&Transform>,
        Query<&Hitbox>,
    )>,
    message_writers: (
        MessageWriter<ChannelChangeMessage>,
        MessageWriter<crate::game::pathfinding::messages::ObstacleChanged>,
    ),
) {
    let (
        infantry_assets,
        archer_assets,
        assassin_assets,
        dispeller_assets,
        shielder_assets,
        healer_assets,
    ) = &unit_assets;
    let (ogre_assets, hag_assets, dark_mage_assets) = &boss_assets;
    let (ref wizard_assets_opt, ref cauldron_assets_opt, mut images) = optional_assets;
    let (
        battlefield_assets,
        spell_visual_assets,
        asset_server,
        flora_assets,
        trampling_grid,
        tree_assets,
        pond_assets,
        bush_assets,
        boulder_assets,
        shadow_assets,
    ) = &shared_assets;
    let (mut channel_change, mut obstacle_events) = message_writers;

    // Process tasks in bulk, breaking only when the next task needs deferred
    // commands from this frame to be flushed first (e.g., Select* tasks need
    // spawned entities to exist, Wizard needs WizardAssets resource).
    // This completes loading in ~4 imperceptible frames instead of hundreds.
    let mut created_deferred_state = false;

    while !spawn_queue.is_complete() {
        let task_ref = spawn_queue.tasks.front().expect("queue not empty");

        // If the next task reads World state and we've written deferred state
        // this frame, break so commands can flush before the next frame.
        if task_ref.needs_command_flush() && created_deferred_state {
            break;
        }

        let task = spawn_queue.tasks.pop_front().expect("queue not empty");
        let creates_state = task.creates_deferred_state();

        match task {
            SpawnTask::DefenderInfantry { unit_index } => {
                infantry_systems::spawn_single_defender(
                    &mut commands,
                    infantry_assets,
                    &mut materials,
                    unit_index,
                );
            }
            SpawnTask::AttackerInfantry { unit_index, level } => {
                infantry_systems::spawn_single_attacker(
                    &mut commands,
                    infantry_assets,
                    &mut materials,
                    unit_index,
                    level,
                );
            }
            SpawnTask::DefenderArcher { unit_index } => {
                archer_systems::spawn_single_defender_archer(
                    &mut commands,
                    archer_assets,
                    &mut materials,
                    unit_index,
                );
            }
            SpawnTask::AttackerArcher { unit_index, level } => {
                archer_systems::spawn_single_attacker_archer(
                    &mut commands,
                    archer_assets,
                    &mut materials,
                    unit_index,
                    level,
                );
            }
            SpawnTask::AttackerAssassin { unit_index, level } => {
                assassin_systems::spawn_single_attacker_assassin(
                    &mut commands,
                    assassin_assets,
                    &mut materials,
                    unit_index,
                    level,
                );
            }
            SpawnTask::AttackerAerialist { unit_index, level } => {
                aerialist_systems::spawn_single_attacker_aerialist(
                    &mut commands,
                    &aerialist_assets,
                    &mut materials,
                    unit_index,
                    level,
                );
            }
            SpawnTask::UpgradeToDispeller { entity } => {
                upgrade_systems::apply_dispeller_upgrade(
                    &mut commands,
                    entity,
                    dispeller_assets,
                    &mut materials,
                );
            }
            SpawnTask::UpgradeToHealer { entity } => {
                upgrade_systems::apply_healer_upgrade(
                    &mut commands,
                    entity,
                    healer_assets,
                    &mut materials,
                );
            }
            SpawnTask::UpgradeToShielder { entity } => {
                upgrade_systems::apply_shielder_upgrade(
                    &mut commands,
                    entity,
                    shielder_assets,
                    &mut materials,
                );
            }
            SpawnTask::King => {
                crate::game::units::king::systems::spawn_king(
                    &mut commands,
                    &king_assets,
                    &mut meshes,
                    &mut materials,
                    &mut king_spawned,
                );
            }
            SpawnTask::KingsGuard { guard_index } => {
                infantry_systems::spawn_single_kings_guard(
                    &mut commands,
                    infantry_assets,
                    &mut materials,
                    guard_index,
                );
            }
            SpawnTask::Brute => {
                crate::game::units::brute::systems::spawn_brute(
                    commands.reborrow(),
                    Res::clone(infantry_assets),
                    &mut materials,
                    Res::clone(&current_level),
                );
            }
            SpawnTask::Ogre => {
                crate::game::units::boss::ogre::systems::spawn_ogre(
                    commands.reborrow(),
                    Res::clone(ogre_assets),
                );
            }
            SpawnTask::Hags => {
                crate::game::units::boss::hags::systems::spawn_hags(
                    commands.reborrow(),
                    Res::clone(hag_assets),
                );
            }
            SpawnTask::DarkMage => {
                crate::game::units::boss::dark_mage::systems::spawn_dark_mage(
                    commands.reborrow(),
                    Res::clone(dark_mage_assets),
                );
            }
            SpawnTask::Battlefield => {
                crate::game::battlefield::systems::setup_battlefield(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    battlefield_assets,
                );
            }
            SpawnTask::Castle => {
                // Castle is spawned as part of battlefield setup
            }
            SpawnTask::LoadWizardAssets => {
                crate::game::units::wizard::systems::load_wizard_assets(
                    commands.reborrow(),
                    Res::clone(asset_server),
                );
            }
            SpawnTask::Wizard => {
                if let Some(assets) = wizard_assets_opt {
                    crate::game::units::wizard::systems::setup_wizard(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &config,
                        assets,
                    );
                }
            }
            SpawnTask::LoadCauldronAssets => {
                crate::game::cauldron::systems::load_cauldron_assets(
                    commands.reborrow(),
                    Res::clone(asset_server),
                );
            }
            SpawnTask::Cauldron => {
                if let Some(assets) = cauldron_assets_opt {
                    crate::game::cauldron::systems::spawn_cauldron(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        assets,
                    );
                }
            }
            SpawnTask::PathfindingGrid => {
                crate::game::pathfinding::systems::initialize_pathfinding(commands.reborrow());
            }
            SpawnTask::SelectInfantryUpgrades => {
                let level = current_level.0;
                let seed_base = level as u64;
                let infantry_attackers: Vec<Entity> = queries
                    .p0()
                    .iter()
                    .filter(|(_, team)| **team == Team::Attackers)
                    .map(|(entity, _)| entity)
                    .collect();
                let selected = upgrade_selection::select_commander_entities(
                    &infantry_attackers,
                    level,
                    MAX_COMMANDER_INFANTRY,
                    seed_base,
                    1,
                    "Infantry",
                );
                spawn_queue.tasks.extend(
                    selected
                        .into_iter()
                        .map(|entity| SpawnTask::UpgradeToCommander { entity }),
                );
            }
            SpawnTask::SelectArcherUpgrades => {
                let level = current_level.0;
                let seed_base = level as u64;
                let archer_attackers: Vec<Entity> = queries
                    .p1()
                    .iter()
                    .filter(|(_, team)| **team == Team::Attackers)
                    .map(|(entity, _)| entity)
                    .collect();
                let selected = upgrade_selection::select_commander_entities(
                    &archer_attackers,
                    level,
                    MAX_COMMANDER_ARCHERS,
                    seed_base,
                    997,
                    "Archer",
                );
                spawn_queue.tasks.extend(
                    selected
                        .into_iter()
                        .map(|entity| SpawnTask::UpgradeToCommander { entity }),
                );
            }
            SpawnTask::SelectDispellerUpgrades => {
                let level = current_level.0;
                let seed_base = level as u64;
                let excluded = collect_excluded_from_tasks(spawn_queue.tasks.make_contiguous());
                let archer_attackers: Vec<Entity> = queries
                    .p1()
                    .iter()
                    .filter(|(_, team)| **team == Team::Attackers)
                    .map(|(entity, _)| entity)
                    .collect();
                let selected = upgrade_selection::select_dispeller_entities(
                    &archer_attackers,
                    level,
                    &excluded,
                    seed_base,
                );
                spawn_queue.tasks.extend(
                    selected
                        .into_iter()
                        .map(|entity| SpawnTask::UpgradeToDispeller { entity }),
                );
            }
            SpawnTask::SelectHealerUpgrades => {
                let level = current_level.0;
                let seed_base = level as u64;
                let excluded = collect_excluded_from_tasks(spawn_queue.tasks.make_contiguous());
                let archer_attackers: Vec<Entity> = queries
                    .p1()
                    .iter()
                    .filter(|(_, team)| **team == Team::Attackers)
                    .map(|(entity, _)| entity)
                    .collect();
                let selected = upgrade_selection::select_healer_entities(
                    &archer_attackers,
                    level,
                    &excluded,
                    seed_base,
                );
                spawn_queue.tasks.extend(
                    selected
                        .into_iter()
                        .map(|entity| SpawnTask::UpgradeToHealer { entity }),
                );
            }
            SpawnTask::SelectShielderUpgrades => {
                let level = current_level.0;
                let seed_base = level as u64;
                let excluded = collect_excluded_from_tasks(spawn_queue.tasks.make_contiguous());
                let infantry_attackers: Vec<Entity> = queries
                    .p0()
                    .iter()
                    .filter(|(_, team)| **team == Team::Attackers)
                    .map(|(entity, _)| entity)
                    .collect();
                let selected = upgrade_selection::select_shielder_entities(
                    &infantry_attackers,
                    level,
                    &excluded,
                    seed_base,
                );
                spawn_queue.tasks.extend(
                    selected
                        .into_iter()
                        .map(|entity| SpawnTask::UpgradeToShielder { entity }),
                );
            }
            SpawnTask::SelectEliteUpgrades => {
                let level = current_level.0;
                let seed_base = level as u64;
                let excluded = collect_excluded_from_tasks(spawn_queue.tasks.make_contiguous());
                // Collect all attacker entities from both queries (ParamSet requires sequential access)
                let mut all_attackers: Vec<Entity> = queries
                    .p0()
                    .iter()
                    .filter(|(_, team)| **team == Team::Attackers)
                    .map(|(entity, _)| entity)
                    .collect();
                all_attackers.extend(
                    queries
                        .p1()
                        .iter()
                        .filter(|(_, team)| **team == Team::Attackers)
                        .map(|(entity, _)| entity),
                );
                let selected = upgrade_selection::select_elite_entities(
                    &all_attackers,
                    level,
                    &excluded,
                    seed_base,
                );
                spawn_queue.tasks.extend(
                    selected
                        .into_iter()
                        .map(|entity| SpawnTask::UpgradeToElite { entity }),
                );
            }
            SpawnTask::UpgradeToElite { entity } => {
                // ParamSet requires sequential access — copy transform before querying hitbox
                if let Ok(transform) = queries.p2().get(entity) {
                    let transform = *transform;
                    if let Ok(hitbox) = queries.p3().get(entity) {
                        let hitbox = *hitbox;
                        upgrade_systems::apply_elite_upgrade(
                            &mut commands,
                            entity,
                            &transform,
                            &hitbox,
                        );
                    }
                }
            }
            SpawnTask::PermanentWall { wall } => {
                crate::game::units::wizard::spells::wall_of_stone::systems::spawn_permanent_wall(
                    &mut commands,
                    spell_visual_assets,
                    &wall,
                );
            }
            SpawnTask::PermanentCrystal {
                crystal,
                damage_mult,
                count_mult,
                resonance_cascade,
            } => {
                crate::game::units::wizard::spells::arcane_crystal::systems::spawn_permanent_crystal(
                    &mut commands,
                    spell_visual_assets,
                    &crystal,
                    damage_mult,
                    count_mult,
                    resonance_cascade,
                );
            }
            SpawnTask::UpgradeToCommander { entity } => {
                // ParamSet requires sequential access — copy transform before querying hitbox
                if let Ok(transform) = queries.p2().get(entity) {
                    let transform = *transform;
                    if let Ok(hitbox) = queries.p3().get(entity) {
                        let hitbox = *hitbox;
                        upgrade_systems::apply_commander_upgrade(
                            &mut commands,
                            entity,
                            &mut materials,
                            &mut meshes,
                            &transform,
                            &hitbox,
                        );
                    }
                }
            }
            SpawnTask::Flora { flora } => {
                crate::game::terrain::flora::systems::spawn_single_flora(
                    &mut commands,
                    flora_assets,
                    shadow_assets,
                    &flora,
                );
            }
            SpawnTask::TerrainBoulder { boulder } => {
                crate::game::terrain::boulder::systems::spawn_terrain_boulder(
                    &mut commands,
                    boulder_assets,
                    shadow_assets,
                    boulder.x,
                    boulder.z,
                    boulder.scale,
                    boulder.sprite_index,
                    &mut obstacle_events,
                );
            }
            SpawnTask::TerrainTree { tree } => {
                crate::game::terrain::tree::systems::spawn_single_tree(
                    &mut commands,
                    tree_assets,
                    shadow_assets,
                    tree.x,
                    tree.z,
                    tree.scale,
                    &mut obstacle_events,
                );
            }
            SpawnTask::TerrainPond { pond } => {
                crate::game::terrain::pond::systems::spawn_single_pond(
                    &mut commands,
                    pond_assets,
                    pond.x,
                    pond.z,
                    pond.radius,
                    &mut obstacle_events,
                );
            }
            SpawnTask::TerrainBush { bush } => {
                crate::game::terrain::bush::systems::spawn_single_bush(
                    &mut commands,
                    bush_assets,
                    shadow_assets,
                    bush.x,
                    bush.z,
                    bush.scale,
                    bush.sprite_index,
                    &mut obstacle_events,
                );
            }
            SpawnTask::TramplingOverlay => {
                crate::game::battlefield::trampling::systems::spawn_trampling_overlay(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut images,
                    trampling_grid,
                );
            }
        }

        if creates_state {
            created_deferred_state = true;
        }
    }

    // Transition to InGame when all tasks are complete
    if spawn_queue.is_complete() {
        channel_change.write(ChannelChangeMessage);
        next_state.set(AppState::InGame);
    }
}

/// Collects entities already targeted for upgrades in existing spawn tasks.
/// Used to avoid double-selecting units across upgrade passes.
fn collect_excluded_from_tasks(tasks: &[SpawnTask]) -> HashSet<Entity> {
    tasks
        .iter()
        .filter_map(|task| match task {
            SpawnTask::UpgradeToElite { entity, .. }
            | SpawnTask::UpgradeToCommander { entity, .. }
            | SpawnTask::UpgradeToDispeller { entity }
            | SpawnTask::UpgradeToHealer { entity }
            | SpawnTask::UpgradeToShielder { entity } => Some(*entity),
            _ => None,
        })
        .collect()
}

/// Cleans up loading resources when exiting loading state.
pub fn cleanup_loading_progress(mut commands: Commands) {
    commands.remove_resource::<SpawnQueue>();
}
