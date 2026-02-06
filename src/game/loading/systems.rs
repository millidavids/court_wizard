use bevy::prelude::*;

use super::resources::LoadingProgress;
use super::spawn_queue::{SpawnQueue, SpawnTask};
use crate::game::constants::*;
use crate::game::resources::CurrentLevel;
use crate::game::units::archer::constants::INITIAL_ARCHER_DEFENDER_COUNT;
use crate::game::units::archer::resources::ArcherAssets;
use crate::game::units::archer::systems as archer_systems;
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::infantry::systems as infantry_systems;
use crate::state::AppState;

/// Initializes the loading progress tracker and spawn queue.
pub fn init_loading_progress(mut commands: Commands, current_level: Res<CurrentLevel>) {
    commands.insert_resource(LoadingProgress::new());

    let mut queue = SpawnQueue::new();
    let level = current_level.0;

    // Spawn in intelligent order: Battlefield -> Castle -> Grid -> King -> Infantry -> Archers -> Behemoth -> Wizard

    // 1. Battlefield (foundation)
    queue.tasks.push(SpawnTask::Battlefield);

    // 2. Castle (part of battlefield)
    queue.tasks.push(SpawnTask::Castle);

    // 3. Pathfinding Grid (needed for unit movement)
    queue.tasks.push(SpawnTask::PathfindingGrid);

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

    // 7. Attacker Infantry
    let total_attackers = calculate_total_infantry(level);
    for i in 0..total_attackers {
        queue.tasks.push(SpawnTask::AttackerInfantry {
            unit_index: i,
            level,
        });
    }

    // 8. Defender Archers
    for i in 0..INITIAL_ARCHER_DEFENDER_COUNT {
        queue
            .tasks
            .push(SpawnTask::DefenderArcher { unit_index: i });
    }

    // 9. Attacker Archers
    let total_attacker_archers = calculate_total_archers(level);
    for i in 0..total_attacker_archers {
        queue.tasks.push(SpawnTask::AttackerArcher {
            unit_index: i,
            level,
        });
    }

    // 10. Behemoth (if level qualifies)
    const BEHEMOTH_SPAWN_LEVEL_INTERVAL: u32 = 3;
    if level >= BEHEMOTH_SPAWN_LEVEL_INTERVAL && level % BEHEMOTH_SPAWN_LEVEL_INTERVAL == 0 {
        queue.tasks.push(SpawnTask::Behemoth);
    }

    // 11. Wizard (last, controls spells)
    queue.tasks.push(SpawnTask::Wizard);

    commands.insert_resource(queue);
}

/// Processes spawn tasks from the queue, spreading them evenly across frames.
#[allow(clippy::too_many_arguments)]
pub fn process_spawn_queue(
    mut commands: Commands,
    mut loading_progress: ResMut<LoadingProgress>,
    mut spawn_queue: ResMut<SpawnQueue>,
    mut next_state: ResMut<NextState<AppState>>,
    // Resources needed for spawning
    infantry_assets: Res<InfantryAssets>,
    archer_assets: Res<ArcherAssets>,
    king_assets: Res<crate::game::units::king::resources::KingAssets>,
    behemoth_assets: Res<crate::game::units::behemoth::resources::BehemothAssets>,
    current_level: Res<CurrentLevel>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    king_spawned: ResMut<crate::game::units::king::components::KingSpawned>,
) {
    // Process exactly one task per frame for smooth, predictable loading
    let batch = spawn_queue.pop_batch(1);

    if let Some(task) = batch.into_iter().next() {
        match task {
            SpawnTask::DefenderInfantry { unit_index } => {
                infantry_systems::spawn_single_defender(
                    &mut commands,
                    &infantry_assets,
                    unit_index,
                );
            }
            SpawnTask::AttackerInfantry { unit_index, level } => {
                infantry_systems::spawn_single_attacker(
                    &mut commands,
                    &infantry_assets,
                    unit_index,
                    level,
                );
            }
            SpawnTask::DefenderArcher { unit_index } => {
                archer_systems::spawn_single_defender_archer(
                    &mut commands,
                    &archer_assets,
                    unit_index,
                );
            }
            SpawnTask::AttackerArcher { unit_index, level } => {
                archer_systems::spawn_single_attacker_archer(
                    &mut commands,
                    &archer_assets,
                    unit_index,
                    level,
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
                    &infantry_assets,
                    guard_index,
                );
            }
            SpawnTask::Behemoth => {
                crate::game::units::behemoth::systems::spawn_initial_behemoths(
                    commands.reborrow(),
                    Res::clone(&behemoth_assets),
                    Res::clone(&current_level),
                );
            }
            SpawnTask::Battlefield => {
                crate::game::battlefield::systems::setup_battlefield(
                    commands.reborrow(),
                    meshes,
                    materials,
                );
            }
            SpawnTask::Castle => {
                // Castle is spawned as part of battlefield setup
            }
            SpawnTask::Wizard => {
                crate::game::units::wizard::systems::setup_wizard(
                    commands.reborrow(),
                    meshes,
                    materials,
                );
            }
            SpawnTask::PathfindingGrid => {
                crate::game::pathfinding::systems::initialize_pathfinding(commands.reborrow());
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
