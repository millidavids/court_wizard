use bevy::prelude::*;

use super::resources::LoadingProgress;
use super::spawn_queue::{SpawnQueue, SpawnTask};
use super::upgrade_selection;
use super::upgrade_systems;
use crate::config::GameConfig;
use crate::game::constants::*;
use crate::game::resources::{CurrentLevel, InitialDefenderCount, KillStats, WaveState};
use crate::game::units::archer::constants::INITIAL_ARCHER_DEFENDER_COUNT;
use crate::game::units::archer::systems as archer_systems;
use crate::game::units::archer::{Archer, ArcherAssets};
use crate::game::units::components::{Hitbox, Team};
use crate::game::battlefield::components::CastleWallAssets;
use crate::game::units::dispeller::DispellerAssets;
use crate::game::units::healer::HealerAssets;
use crate::game::units::infantry::Infantry;
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::infantry::systems as infantry_systems;
use crate::state::AppState;

/// Initializes the loading progress tracker and spawn queue.
pub fn init_loading_progress(
    mut commands: Commands,
    mut current_level: ResMut<CurrentLevel>,
    mut kill_stats: ResMut<KillStats>,
    config: Res<GameConfig>,
) {
    // Sync CurrentLevel from GameConfig (may have been updated by save loading)
    current_level.0 = config.current_level;

    commands.insert_resource(LoadingProgress::new());

    let mut queue = SpawnQueue::new();
    let level = current_level.0;

    // Spawn in intelligent order: Battlefield -> Castle -> Grid -> King -> Infantry -> Archers -> Brute/Ogre -> Wizard

    // 1. Battlefield (foundation)
    queue.tasks.push(SpawnTask::Battlefield);

    // 2. Castle (part of battlefield)
    queue.tasks.push(SpawnTask::Castle);

    // 3. Pathfinding Grid (needed for unit movement)
    queue.tasks.push(SpawnTask::PathfindingGrid);

    // 3b. Permanent walls from previous victories (after pathfinding grid)
    for saved_wall in &config.saved_walls {
        queue.tasks.push(SpawnTask::PermanentWall {
            wall: saved_wall.clone(),
        });
    }

    // 4. King (central defender)
    queue.tasks.push(SpawnTask::King);

    // 5. King's Guard (protect the king)
    for i in 0..KINGS_GUARD_COUNT {
        queue.tasks.push(SpawnTask::KingsGuard { guard_index: i });
    }

    // 6. Defender Infantry
    for i in 0..INITIAL_DEFENDER_COUNT {
        queue
            .tasks
            .push(SpawnTask::DefenderInfantry { unit_index: i });
    }

    // Check if this is a boss level (every 5th level starting at 5)
    use crate::game::constants::is_boss_level;
    use crate::game::constants::get_tier;
    use crate::game::units::brute::constants::BRUTE_START_TIER;

    if is_boss_level(level) {
        // Boss level: only spawn the ogre, no other attackers, no wave system
        queue.tasks.push(SpawnTask::Ogre);
        kill_stats.total_attackers_spawned = 1;
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

        // 7. Attacker Infantry (wave 1)
        let total_attackers = calculate_total_infantry(level);
        for i in 0..total_attackers {
            queue.tasks.push(SpawnTask::AttackerInfantry {
                unit_index: i,
                level,
            });
        }

        // 8. Attacker Archers (wave 1)
        let total_attacker_archers = calculate_total_archers(level);
        for i in 0..total_attacker_archers {
            queue.tasks.push(SpawnTask::AttackerArcher {
                unit_index: i,
                level,
            });
        }

        // 9. Brute (if tier qualifies, wave 1)
        let has_brute = get_tier(level) >= BRUTE_START_TIER;
        if has_brute {
            queue.tasks.push(SpawnTask::Brute);
        }

        // Record total attackers spawned across ALL waves for score screen
        let per_wave = total_attackers + total_attacker_archers + if has_brute { 1 } else { 0 };
        kill_stats.total_attackers_spawned = per_wave * wave_count;

        // Initialize wave state
        commands.insert_resource(WaveState {
            current_wave: 0,
            total_waves: wave_count,
            wave_timer: WAVE_INTERVAL_SECONDS,
            wave_interval: WAVE_INTERVAL_SECONDS,
            waves_complete: wave_count <= 1,
        });
    }

    // 8. Defender Archers (always spawn regardless of boss level)
    for i in 0..INITIAL_ARCHER_DEFENDER_COUNT {
        queue
            .tasks
            .push(SpawnTask::DefenderArcher { unit_index: i });
    }

    // Track initial defender count for spell shield threshold (multiplayer)
    commands.insert_resource(InitialDefenderCount(
        INITIAL_DEFENDER_COUNT + KINGS_GUARD_COUNT + INITIAL_ARCHER_DEFENDER_COUNT,
    ));

    // 11. Load wizard assets (sprite sheet texture)
    queue.tasks.push(SpawnTask::LoadWizardAssets);

    // 12. Wizard (controls spells)
    queue.tasks.push(SpawnTask::Wizard);

    // 13. Load cauldron assets (texture for sprite sheet)
    queue.tasks.push(SpawnTask::LoadCauldronAssets);

    // 13. Cauldron (next to wizard on castle wall)
    queue.tasks.push(SpawnTask::Cauldron);

    // 14. Select upgrades for attacker units (after all attackers spawn)
    queue.tasks.push(SpawnTask::SelectInfantryUpgrades);
    queue.tasks.push(SpawnTask::SelectArcherUpgrades);
    queue.tasks.push(SpawnTask::SelectDispellerUpgrades);
    queue.tasks.push(SpawnTask::SelectHealerUpgrades);

    commands.insert_resource(queue);
}

/// Processes spawn tasks from the queue, spreading them evenly across frames.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn process_spawn_queue(
    mut commands: Commands,
    mut loading_progress: ResMut<LoadingProgress>,
    mut spawn_queue: ResMut<SpawnQueue>,
    mut next_state: ResMut<NextState<AppState>>,
    // Resources needed for spawning
    config: Res<GameConfig>,
    unit_assets: (Res<InfantryAssets>, Res<ArcherAssets>, Res<DispellerAssets>, Res<HealerAssets>),
    king_assets: Res<crate::game::units::king::resources::KingAssets>,
    attacker_assets: (
        Res<crate::game::units::brute::resources::BruteAssets>,
        Res<crate::game::units::boss::ogre::resources::OgreAssets>,
    ),
    current_level: Res<CurrentLevel>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    king_spawned: ResMut<crate::game::units::king::components::KingSpawned>,
    optional_assets: (
        Option<Res<crate::game::units::wizard::components::WizardAssets>>,
        Option<Res<crate::game::cauldron::resources::CauldronAssets>>,
    ),
    shared_assets: (
        Res<CastleWallAssets>,
        Res<crate::game::units::wizard::spells::visual_assets::SpellVisualAssets>,
        Res<AssetServer>,
    ),
    // Use ParamSet to reduce parameter count and avoid query conflicts
    mut queries: ParamSet<(
        Query<(Entity, &Team), With<Infantry>>,
        Query<(Entity, &Team), With<Archer>>,
        Query<&Transform>,
        Query<&Hitbox>,
    )>,
) {
    // Process exactly one task per frame for smooth, predictable loading
    let batch = spawn_queue.pop_batch(1);

    if let Some(task) = batch.into_iter().next() {
        match task {
            SpawnTask::DefenderInfantry { unit_index } => {
                infantry_systems::spawn_single_defender(&mut commands, &unit_assets.0, unit_index);
            }
            SpawnTask::AttackerInfantry { unit_index, level } => {
                infantry_systems::spawn_single_attacker(
                    &mut commands,
                    &unit_assets.0,
                    unit_index,
                    level,
                );
            }
            SpawnTask::DefenderArcher { unit_index } => {
                archer_systems::spawn_single_defender_archer(
                    &mut commands,
                    &unit_assets.1,
                    unit_index,
                );
            }
            SpawnTask::AttackerArcher { unit_index, level } => {
                archer_systems::spawn_single_attacker_archer(
                    &mut commands,
                    &unit_assets.1,
                    unit_index,
                    level,
                );
            }
            SpawnTask::UpgradeToDispeller { entity } => {
                upgrade_systems::apply_dispeller_upgrade(
                    &mut commands,
                    entity,
                    &unit_assets.2,
                );
            }
            SpawnTask::UpgradeToHealer { entity } => {
                upgrade_systems::apply_healer_upgrade(
                    &mut commands,
                    entity,
                    &unit_assets.3,
                );
            }
            SpawnTask::King => {
                crate::game::units::king::systems::spawn_king(
                    commands.reborrow(),
                    Res::clone(&king_assets),
                    meshes,
                    materials,
                    king_spawned,
                );
            }
            SpawnTask::KingsGuard { guard_index } => {
                infantry_systems::spawn_single_kings_guard(
                    &mut commands,
                    &unit_assets.0,
                    guard_index,
                );
            }
            SpawnTask::Brute => {
                crate::game::units::brute::systems::spawn_brute(
                    commands.reborrow(),
                    Res::clone(&attacker_assets.0),
                    Res::clone(&current_level),
                );
            }
            SpawnTask::Ogre => {
                crate::game::units::boss::ogre::systems::spawn_ogre(
                    commands.reborrow(),
                    Res::clone(&attacker_assets.1),
                );
            }
            SpawnTask::Battlefield => {
                crate::game::battlefield::systems::setup_battlefield(
                    commands.reborrow(),
                    meshes,
                    materials,
                    Res::clone(&shared_assets.0),
                );
            }
            SpawnTask::Castle => {
                // Castle is spawned as part of battlefield setup
            }
            SpawnTask::LoadWizardAssets => {
                crate::game::units::wizard::systems::load_wizard_assets(
                    commands.reborrow(),
                    Res::clone(&shared_assets.2),
                );
            }
            SpawnTask::Wizard => {
                if let Some(ref assets) = optional_assets.0 {
                    crate::game::units::wizard::systems::setup_wizard(
                        commands.reborrow(),
                        meshes,
                        materials,
                        Res::clone(&config),
                        Res::clone(assets),
                    );
                }
            }
            SpawnTask::LoadCauldronAssets => {
                crate::game::cauldron::systems::load_cauldron_assets(
                    commands.reborrow(),
                    Res::clone(&shared_assets.2),
                );
            }
            SpawnTask::Cauldron => {
                if let Some(ref assets) = optional_assets.1 {
                    crate::game::cauldron::systems::spawn_cauldron(
                        commands.reborrow(),
                        meshes,
                        materials,
                        Res::clone(assets),
                    );
                }
            }
            SpawnTask::PathfindingGrid => {
                crate::game::pathfinding::systems::initialize_pathfinding(commands.reborrow());
            }
            SpawnTask::SelectInfantryUpgrades => {
                let level = current_level.0;
                let upgrade_tasks =
                    upgrade_selection::select_infantry_upgrades(&queries.p0(), level);
                spawn_queue.tasks.extend(upgrade_tasks);
            }
            SpawnTask::SelectArcherUpgrades => {
                let level = current_level.0;
                let upgrade_tasks = upgrade_selection::select_archer_upgrades(&queries.p1(), level);
                spawn_queue.tasks.extend(upgrade_tasks);
            }
            SpawnTask::SelectDispellerUpgrades => {
                let level = current_level.0;
                let upgrade_tasks = upgrade_selection::select_dispeller_upgrades(
                    &queries.p1(),
                    level,
                    &spawn_queue.tasks,
                );
                spawn_queue.tasks.extend(upgrade_tasks);
            }
            SpawnTask::SelectHealerUpgrades => {
                let level = current_level.0;
                let upgrade_tasks = upgrade_selection::select_healer_upgrades(
                    &queries.p1(),
                    level,
                    &spawn_queue.tasks,
                );
                spawn_queue.tasks.extend(upgrade_tasks);
            }
            SpawnTask::UpgradeToElite { entity, unit_type } => {
                // Query the entity's current transform and hitbox (query separately to avoid double borrow)
                if let Ok(transform) = queries.p2().get(entity) {
                    let transform = *transform; // Copy the transform
                    if let Ok(hitbox) = queries.p3().get(entity) {
                        let hitbox = *hitbox; // Copy the hitbox
                        upgrade_systems::apply_elite_upgrade(
                            &mut commands,
                            entity,
                            unit_type,
                            &mut materials,
                            &transform,
                            &hitbox,
                        );
                    }
                }
            }
            SpawnTask::PermanentWall { wall } => {
                crate::game::units::wizard::spells::wall_of_stone::systems::spawn_permanent_wall(
                    &mut commands,
                    &shared_assets.1,
                    &wall,
                );
            }
            SpawnTask::UpgradeToCommander { entity, unit_type } => {
                // Query the entity's current transform and hitbox (query separately to avoid double borrow)
                if let Ok(transform) = queries.p2().get(entity) {
                    let transform = *transform; // Copy the transform
                    if let Ok(hitbox) = queries.p3().get(entity) {
                        let hitbox = *hitbox; // Copy the hitbox
                        upgrade_systems::apply_commander_upgrade(
                            &mut commands,
                            entity,
                            unit_type,
                            &mut materials,
                            &mut meshes,
                            &transform,
                            &hitbox,
                        );
                    }
                }
            }
        }
    }

    // Advance to next frame
    loading_progress.advance();

    // Transition to InGame when all tasks are complete
    if spawn_queue.is_complete() && loading_progress.is_complete() {
        next_state.set(AppState::InGame);
    }
}

/// Cleans up the loading progress resource when exiting loading state.
pub fn cleanup_loading_progress(mut commands: Commands) {
    commands.remove_resource::<LoadingProgress>();
    commands.remove_resource::<SpawnQueue>();
}
