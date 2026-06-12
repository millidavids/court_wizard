//! Multiplayer loading initialisation: builds the deterministic spawn queue.
//!
//! Sets `GameConfig.seed` to the host-shared seed, runs the same deterministic
//! terrain generators single-player uses, and enqueues one spawn task per
//! generated terrain element — so both peers produce an identical world.

use bevy::prelude::*;

use super::queue::{MpSpawnQueue, MpSpawnTask};
use super::resources::{MpConfigBackup, MpLoadingSync};
use crate::config::GameConfig;
use crate::game::constants::*;
use crate::game::seeded_rng::resources::GameSeed;
use crate::networking::resources::PeerRole;
use crate::networking::session::MultiplayerSession;

/// Initializes the multiplayer loading spawn queue.
///
/// Sets `GameConfig.seed` to the host-shared seed, runs the same deterministic
/// terrain generators single-player uses, and enqueues one spawn task per
/// generated terrain element — so both peers produce an identical world.
pub fn init_mp_loading(
    mut commands: Commands,
    session: Res<MultiplayerSession>,
    game_seed: Res<GameSeed>,
    coop_level: Option<Res<crate::game::multiplayer::coop::CoopGuestLevel>>,
    mut config: ResMut<GameConfig>,
) {
    commands.insert_resource(MpLoadingSync::default());

    // Stash the SP seed/level so MP exit can restore them.
    commands.insert_resource(MpConfigBackup {
        previous_seed: config.seed,
        previous_current_level: config.current_level,
    });

    // Co-op builds the SINGLE-PLAYER battlefield shell (one castle, no armies —
    // the host streams its army as ghosts), at the HOST's level so the seeded
    // terrain matches the host's SP loader, and WITHOUT the mirrored forest.
    // Versus builds the mirrored two-castle arena at a fixed terrain level.
    let coop = session.is_coop();
    let terrain_level = if coop {
        coop_level.map(|l| l.0).unwrap_or(MP_TERRAIN_LEVEL)
    } else {
        MP_TERRAIN_LEVEL
    };

    // Seed the deterministic terrain generators with the host-shared seed.
    config.seed = Some(game_seed.0);
    // `generate_flora_positions` reads `config.current_level` for its seed
    // derivation (not a parameter), so we MUST override it here too. Otherwise
    // each peer would use its own single-player progression level and the two
    // clients would generate different flora — which cascades into different
    // boulder/tree/bush/pond placements (those generators consult saved_flora
    // for obstacle avoidance).
    config.current_level = terrain_level;

    // Wipe any cached terrain (left over from a prior single-player run) so
    // the generators produce fresh content keyed off the MP seed.
    config.saved_flora.clear();
    config.saved_trees.clear();
    config.saved_ponds.clear();
    config.saved_bushes.clear();
    config.saved_boulders.clear();

    // Generate terrain identically on both peers — both call the same
    // seeded RNG path that single-player uses, independent of `GameRng`.
    crate::game::terrain::flora::systems::generate_flora_positions(&mut config);
    crate::game::loading::terrain_generation::generate_terrain(
        &mut config,
        terrain_level,
        1.0,
        !coop, // versus mirrors the forest band; co-op uses the SP single-side layout
    );

    let mut queue = MpSpawnQueue::new();

    // Static world.
    queue.tasks.push(MpSpawnTask::Battlefield);
    // Co-op shares the host's single castle; only versus has the opposite-corner
    // Castle 2.
    if !coop {
        queue.tasks.push(MpSpawnTask::Castle2);
    }
    queue.tasks.push(MpSpawnTask::PathfindingGrid);

    // Terrain — one task per element, mirroring single-player's queue layout.
    for boulder in &config.saved_boulders {
        queue.tasks.push(MpSpawnTask::TerrainBoulder {
            boulder: boulder.clone(),
        });
    }
    for tree in &config.saved_trees {
        queue
            .tasks
            .push(MpSpawnTask::TerrainTree { tree: tree.clone() });
    }
    for pond in &config.saved_ponds {
        queue
            .tasks
            .push(MpSpawnTask::TerrainPond { pond: pond.clone() });
    }
    for bush in &config.saved_bushes {
        queue
            .tasks
            .push(MpSpawnTask::TerrainBush { bush: bush.clone() });
    }
    for flora in &config.saved_flora {
        queue.tasks.push(MpSpawnTask::Flora {
            flora: flora.clone(),
        });
    }

    // Host spawns all gameplay entities (both armies). Guest receives them
    // via state snapshots once the match starts.
    if session.role == PeerRole::Host {
        queue.tasks.push(MpSpawnTask::HostKing);
        for i in 0..KINGS_GUARD_COUNT {
            queue
                .tasks
                .push(MpSpawnTask::HostKingsGuard { guard_index: i });
        }
        queue.tasks.push(MpSpawnTask::GuestKing);
        for i in 0..KINGS_GUARD_COUNT {
            queue
                .tasks
                .push(MpSpawnTask::GuestKingsGuard { guard_index: i });
        }
        for i in 0..MP_INFANTRY_COUNT {
            queue
                .tasks
                .push(MpSpawnTask::HostInfantry { unit_index: i });
        }
        for i in 0..MP_ARCHER_COUNT {
            queue.tasks.push(MpSpawnTask::HostArcher { unit_index: i });
        }
        for i in 0..MP_INFANTRY_COUNT {
            queue
                .tasks
                .push(MpSpawnTask::GuestInfantry { unit_index: i });
        }
        for i in 0..MP_ARCHER_COUNT {
            queue.tasks.push(MpSpawnTask::GuestArcher { unit_index: i });
        }
    }

    queue.tasks.push(MpSpawnTask::LoadWizardAssets);
    queue.tasks.push(MpSpawnTask::HostWizard);
    queue.tasks.push(MpSpawnTask::GuestWizard);
    queue.tasks.push(MpSpawnTask::LoadCauldronAssets);
    queue.tasks.push(MpSpawnTask::Cauldron);

    commands.insert_resource(queue);
}
