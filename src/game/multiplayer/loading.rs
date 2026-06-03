//! Multiplayer loading systems.
//!
//! Builds and processes a multiplayer-specific spawn queue. Both peers spawn
//! the full single-player battlefield and the same deterministic terrain
//! (driven by the host-shared seed), so each client sees the same world. The
//! host additionally enqueues all gameplay entities (kings, guards, infantry,
//! archers); the guest receives those via state snapshots.

use bevy::prelude::*;

use super::spawning::*;
use crate::config::GameConfig;
use crate::config::save_data::{SavedBoulder, SavedBush, SavedFlora, SavedPond, SavedTree};
use crate::game::battlefield::components::BattlefieldAssets;
use crate::game::battlefield::ground_material::{GroundMaterial, StoneNoiseMaterial};
use crate::game::constants::*;
use crate::game::pathfinding::messages::ObstacleChanged;
use crate::game::seeded_rng::resources::{GameRng, GameSeed};
use crate::game::shared_systems::ShadowAssets;
use crate::game::terrain::boulder::resources::BoulderAssets;
use crate::game::terrain::bush::resources::BushAssets;
use crate::game::terrain::flora::resources::FloraAssets;
use crate::game::terrain::pond::resources::PondAssets;
use crate::game::terrain::tree::resources::TreeAssets;
use crate::game::units::archer::ArcherAssets;
use crate::game::units::components::Team;
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::king::components::KingSpawned;
use crate::game::units::wizard::components::*;
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{NetworkConnection, PeerRole};
use crate::networking::session::MultiplayerSession;
use crate::state::AppState;

/// Multiplayer-specific spawn tasks.
pub enum MpSpawnTask {
    /// Build the full single-player battlefield (tiled ground, castle #1,
    /// left/right wall backdrops, wall-floor, stone/sand underlays, lava
    /// pool, water-ripple mesh).
    Battlefield,
    /// Spawn the second castle (Castle 1 is spawned by `setup_battlefield`).
    Castle2,
    /// Initialize the pathfinding grid.
    PathfindingGrid,
    /// Load wizard sprite sheet assets.
    LoadWizardAssets,
    HostWizard,
    GuestWizard,
    HostInfantry {
        unit_index: u32,
    },
    HostArcher {
        unit_index: u32,
    },
    GuestInfantry {
        unit_index: u32,
    },
    GuestArcher {
        unit_index: u32,
    },
    HostKing,
    HostKingsGuard {
        guard_index: u32,
    },
    GuestKing,
    GuestKingsGuard {
        guard_index: u32,
    },
    /// A single flora decoration (visual only).
    Flora {
        flora: SavedFlora,
    },
    /// A single boulder (pathfinding obstacle).
    TerrainBoulder {
        boulder: SavedBoulder,
    },
    /// A single tree (pathfinding obstacle).
    TerrainTree {
        tree: SavedTree,
    },
    /// A single pond (slow terrain).
    TerrainPond {
        pond: SavedPond,
    },
    /// A single bush (slow terrain).
    TerrainBush {
        bush: SavedBush,
    },
    /// Load the cauldron sprite-sheet asset (must precede `Cauldron`).
    LoadCauldronAssets,
    /// Spawn the cauldron entity so brewing works in multiplayer.
    Cauldron,
}

impl MpSpawnTask {
    /// Tasks that read a resource inserted by a prior task via
    /// `commands.insert_resource(...)`. The processor must flush the
    /// command queue (end the frame) before running these — Bevy doesn't
    /// apply deferred resource inserts until the next sync point.
    fn needs_command_flush(&self) -> bool {
        matches!(
            self,
            MpSpawnTask::HostWizard | MpSpawnTask::GuestWizard | MpSpawnTask::Cauldron,
        )
    }

    /// Tasks that schedule a deferred `commands.insert_resource(...)` that
    /// a subsequent task will need to read.
    fn creates_deferred_state(&self) -> bool {
        matches!(
            self,
            MpSpawnTask::LoadWizardAssets | MpSpawnTask::LoadCauldronAssets,
        )
    }
}

/// Resource that holds the multiplayer spawn queue.
#[derive(Resource)]
pub struct MpSpawnQueue {
    pub tasks: Vec<MpSpawnTask>,
}

impl MpSpawnQueue {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn pop_next(&mut self) -> Option<MpSpawnTask> {
        if self.tasks.is_empty() {
            None
        } else {
            Some(self.tasks.remove(0))
        }
    }

    pub fn is_complete(&self) -> bool {
        self.tasks.is_empty()
    }
}

/// Tracks whether both players have finished loading.
#[derive(Resource, Default)]
pub struct MpLoadingSync {
    pub my_loaded: bool,
    pub peer_loaded: bool,
}

/// Snapshot of the `GameConfig` fields multiplayer overwrites during loading,
/// so MP exit can restore them and not pollute later single-player runs.
#[derive(Resource)]
pub struct MpConfigBackup {
    pub previous_seed: Option<u64>,
    pub previous_current_level: u32,
}

/// Initializes the multiplayer loading spawn queue.
///
/// Sets `GameConfig.seed` to the host-shared seed, runs the same deterministic
/// terrain generators single-player uses, and enqueues one spawn task per
/// generated terrain element — so both peers produce an identical world.
pub fn init_mp_loading(
    mut commands: Commands,
    session: Res<MultiplayerSession>,
    game_seed: Res<GameSeed>,
    mut config: ResMut<GameConfig>,
) {
    commands.insert_resource(MpLoadingSync::default());

    // Stash the SP seed/level so MP exit can restore them.
    commands.insert_resource(MpConfigBackup {
        previous_seed: config.seed,
        previous_current_level: config.current_level,
    });

    // Seed the deterministic terrain generators with the host-shared seed.
    config.seed = Some(game_seed.0);
    // `generate_flora_positions` reads `config.current_level` for its seed
    // derivation (not a parameter), so we MUST override it here too. Otherwise
    // each peer would use its own single-player progression level and the two
    // clients would generate different flora — which cascades into different
    // boulder/tree/bush/pond placements (those generators consult saved_flora
    // for obstacle avoidance).
    config.current_level = MP_TERRAIN_LEVEL;

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
        MP_TERRAIN_LEVEL,
        1.0,
        true, // multiplayer: mirror the forest band to the opposite side
    );

    let mut queue = MpSpawnQueue::new();

    // Static world.
    queue.tasks.push(MpSpawnTask::Battlefield);
    queue.tasks.push(MpSpawnTask::Castle2);
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

/// Processes one multiplayer spawn task per frame, then handles the
/// "both peers loaded" handshake.
#[allow(clippy::too_many_arguments)]
pub fn process_mp_spawn_queue(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    session: Res<MultiplayerSession>,
    mut next_state: ResMut<NextState<AppState>>,
    progress: (ResMut<MpSpawnQueue>, ResMut<MpLoadingSync>),
    unit_assets: (
        Res<InfantryAssets>,
        Res<ArcherAssets>,
        Res<crate::game::units::king::resources::KingAssets>,
        ResMut<KingSpawned>,
    ),
    terrain_assets: (
        Res<BoulderAssets>,
        Res<TreeAssets>,
        Res<PondAssets>,
        Res<BushAssets>,
        Res<FloraAssets>,
        Res<ShadowAssets>,
    ),
    battlefield: (
        Res<BattlefieldAssets>,
        ResMut<Assets<GroundMaterial>>,
        ResMut<Assets<StoneNoiseMaterial>>,
        ResMut<GameRng>,
    ),
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    wizard_assets: Option<Res<WizardAssets>>,
    cauldron_assets: Option<Res<crate::game::cauldron::resources::CauldronAssets>>,
    spell_assets: Res<crate::game::units::wizard::spells::visual_assets::SpellVisualAssets>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let (mut spawn_queue, mut loading_sync) = progress;
    let (infantry_assets, archer_assets, king_assets, mut king_spawned) = unit_assets;
    let (boulder_assets, tree_assets, pond_assets, bush_assets, flora_assets, shadow_assets) =
        terrain_assets;
    let (battlefield_assets, mut ground_materials, mut stone_materials, mut game_rng) = battlefield;

    // Per-client visual mirror: the battlefield/walls/lava/water are purely
    // visual and rendered locally — units and terrain stay in shared world
    // coords. The guest spawns its copy rotated 180° around the world origin
    // so that, combined with its mirrored camera, the asymmetric SP wall art
    // (right wall with tunnels, left wall) appears correctly oriented from
    // the guest's perspective.
    let origin_transform = if session.role == PeerRole::Guest {
        Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI))
    } else {
        Transform::IDENTITY
    };

    // Bulk-process tasks each frame — matches single-player's `process_spawn_queue`.
    // The break-on-deferred-state logic lets us still respect tasks that need
    // a previous task's `commands.insert_resource(...)` to have flushed
    // (HostWizard/GuestWizard need WizardAssets; Cauldron needs CauldronAssets).
    // Without this, the entire MP arena materialises in ~3 frames instead of ~100,
    // small enough that no loading screen is needed.
    let mut created_deferred_state = false;
    while let Some(task_ref) = spawn_queue.tasks.first() {
        if task_ref.needs_command_flush() && created_deferred_state {
            break;
        }
        let Some(task) = spawn_queue.pop_next() else {
            break;
        };
        if task.creates_deferred_state() {
            created_deferred_state = true;
        }
        match task {
            MpSpawnTask::Battlefield => {
                // Reuse single-player's battlefield builder verbatim. Both
                // peers draw ground-tile RNG from the shared `GameRng`, so
                // tile placement is identical; the guest's `origin_transform`
                // mirrors the result to match its camera.
                crate::game::battlefield::systems::setup_battlefield(
                    &mut game_rng.0,
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut ground_materials,
                    &mut stone_materials,
                    &battlefield_assets,
                    origin_transform,
                );
            }
            MpSpawnTask::Castle2 => {
                spawn_castle(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &battlefield_assets,
                    CASTLE_2_POSITION,
                    CASTLE_2_ROTATION_DEGREES,
                    origin_transform,
                );
            }
            MpSpawnTask::PathfindingGrid => {
                crate::game::pathfinding::systems::initialize_pathfinding(commands.reborrow());
            }
            MpSpawnTask::LoadWizardAssets => {
                crate::game::units::wizard::systems::load_wizard_assets(
                    commands.reborrow(),
                    Res::clone(&asset_server),
                );
            }
            MpSpawnTask::HostWizard => {
                let Some(ref assets) = wizard_assets else {
                    // `WizardAssets` was scheduled via deferred
                    // `commands.insert_resource(...)` by LoadWizardAssets
                    // and hasn't flushed yet. Re-push to the front and
                    // bail so we retry next frame instead of silently
                    // consuming the task and leaving the player with no
                    // wizard.
                    spawn_queue.tasks.insert(0, MpSpawnTask::HostWizard);
                    break;
                };
                spawn_mp_wizard(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    WIZARD_POSITION,
                    session.host_wizard,
                    session.role,
                    true,
                    assets,
                );
            }
            MpSpawnTask::GuestWizard => {
                let Some(ref assets) = wizard_assets else {
                    spawn_queue.tasks.insert(0, MpSpawnTask::GuestWizard);
                    break;
                };
                spawn_mp_wizard(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    WIZARD_2_POSITION,
                    session.guest_wizard,
                    session.role,
                    false,
                    assets,
                );
            }
            MpSpawnTask::HostInfantry { unit_index } => {
                spawn_mp_infantry(
                    &mut commands,
                    &infantry_assets,
                    &mut materials,
                    unit_index,
                    Team::Defenders,
                    true,
                );
            }
            MpSpawnTask::GuestInfantry { unit_index } => {
                spawn_mp_infantry(
                    &mut commands,
                    &infantry_assets,
                    &mut materials,
                    unit_index,
                    Team::Attackers,
                    false,
                );
            }
            MpSpawnTask::HostArcher { unit_index } => {
                spawn_mp_archer(
                    &mut commands,
                    &archer_assets,
                    &mut materials,
                    unit_index,
                    Team::Defenders,
                    true,
                );
            }
            MpSpawnTask::GuestArcher { unit_index } => {
                spawn_mp_archer(
                    &mut commands,
                    &archer_assets,
                    &mut materials,
                    unit_index,
                    Team::Attackers,
                    false,
                );
            }
            MpSpawnTask::HostKing => {
                spawn_mp_king(
                    &mut commands,
                    &king_assets,
                    &spell_assets,
                    &mut materials,
                    &mut king_spawned,
                    WIZARD_POSITION,
                    DEFENDER_GRID_CENTER_ANGLE,
                    Team::Defenders,
                );
            }
            MpSpawnTask::GuestKing => {
                let mirrored_angle = DEFENDER_GRID_CENTER_ANGLE + std::f32::consts::PI;
                spawn_mp_king(
                    &mut commands,
                    &king_assets,
                    &spell_assets,
                    &mut materials,
                    &mut king_spawned,
                    WIZARD_2_POSITION,
                    mirrored_angle,
                    Team::Attackers,
                );
            }
            MpSpawnTask::HostKingsGuard { guard_index } => {
                spawn_mp_kings_guard(
                    &mut commands,
                    &infantry_assets,
                    &mut materials,
                    guard_index,
                    WIZARD_POSITION,
                    DEFENDER_GRID_CENTER_ANGLE,
                    Team::Defenders,
                );
            }
            MpSpawnTask::GuestKingsGuard { guard_index } => {
                let mirrored_angle = DEFENDER_GRID_CENTER_ANGLE + std::f32::consts::PI;
                spawn_mp_kings_guard(
                    &mut commands,
                    &infantry_assets,
                    &mut materials,
                    guard_index,
                    WIZARD_2_POSITION,
                    mirrored_angle,
                    Team::Attackers,
                );
            }
            MpSpawnTask::Flora { flora } => {
                crate::game::terrain::flora::systems::spawn_single_flora(
                    &mut commands,
                    &flora_assets,
                    &shadow_assets,
                    &flora,
                );
            }
            MpSpawnTask::TerrainBoulder { boulder } => {
                crate::game::terrain::boulder::systems::spawn_terrain_boulder(
                    &mut commands,
                    &boulder_assets,
                    &shadow_assets,
                    boulder.x,
                    boulder.z,
                    boulder.scale,
                    boulder.sprite_index,
                    &mut obstacle_events,
                );
            }
            MpSpawnTask::TerrainTree { tree } => {
                crate::game::terrain::tree::systems::spawn_single_tree(
                    &mut commands,
                    &tree_assets,
                    &shadow_assets,
                    tree.x,
                    tree.z,
                    tree.scale,
                    tree.sprite_index,
                    &mut obstacle_events,
                );
            }
            MpSpawnTask::TerrainPond { pond } => {
                crate::game::terrain::pond::systems::spawn_single_pond(
                    &mut commands,
                    &pond_assets,
                    pond.x,
                    pond.z,
                    pond.radius,
                    &mut obstacle_events,
                );
            }
            MpSpawnTask::TerrainBush { bush } => {
                crate::game::terrain::bush::systems::spawn_single_bush(
                    &mut commands,
                    &bush_assets,
                    &shadow_assets,
                    bush.x,
                    bush.z,
                    bush.scale,
                    bush.sprite_index,
                    &mut obstacle_events,
                );
            }
            MpSpawnTask::LoadCauldronAssets => {
                crate::game::cauldron::systems::load_cauldron_assets(
                    commands.reborrow(),
                    Res::clone(&asset_server),
                );
            }
            MpSpawnTask::Cauldron => {
                let Some(assets) = cauldron_assets.as_deref() else {
                    // `CauldronAssets` was scheduled by LoadCauldronAssets
                    // and hasn't flushed yet. Re-push and bail so we retry
                    // next frame — silently consuming this task would leave
                    // the match with no cauldron, permanently breaking
                    // brewing for the rest of the game.
                    spawn_queue.tasks.insert(0, MpSpawnTask::Cauldron);
                    break;
                };
                // Place the cauldron beside the LOCAL wizard: host at
                // CAULDRON_POSITION, guest at the mirrored CAULDRON_2_POSITION
                // (otherwise the guest's cauldron sits at the host's castle).
                let cauldron_pos = if session.role == PeerRole::Guest {
                    crate::game::cauldron::constants::CAULDRON_2_POSITION
                } else {
                    crate::game::cauldron::constants::CAULDRON_POSITION
                };
                crate::game::cauldron::systems::spawn_cauldron(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    assets,
                    cauldron_pos,
                );
            }
        }
    }

    // As soon as the spawn queue empties, tell the peer we're ready. The
    // previous code waited for an artificial 100-frame floor (~1.7s) so the
    // loading screen had something to show; with the bulk processor above,
    // both peers complete in ~3 imperceptible frames and the screen is gone.
    if spawn_queue.is_complete() && !loading_sync.my_loaded {
        loading_sync.my_loaded = true;
        connection
            .outgoing_messages
            .push(NetworkMessage::GameLoaded);
    }

    // Check for peer's GameLoaded message
    let has_game_loaded = connection
        .incoming_messages
        .iter()
        .any(|m| matches!(m, NetworkMessage::GameLoaded));

    if has_game_loaded {
        connection
            .incoming_messages
            .retain(|m| !matches!(m, NetworkMessage::GameLoaded));
        loading_sync.peer_loaded = true;
    }

    // Both loaded — transition to gameplay. Guard against a same-frame
    // disconnect: if `detect_mp_loading_disconnect` has just set NextState
    // back to MetaGame, we must NOT clobber that transition by jumping
    // into MultiplayerGame on a dead connection. Checking
    // `connection.state == Connected` short-circuits the race.
    if loading_sync.my_loaded
        && loading_sync.peer_loaded
        && connection.state == crate::networking::resources::ConnectionState::Connected
    {
        next_state.set(AppState::MultiplayerGame);
    }
}

/// Cleans up multiplayer loading resources and restores any `GameConfig`
/// fields multiplayer overwrote — runs on every exit of `MultiplayerLoading`
/// (success or abort), so the seed/level/saved-terrain mutations never leak
/// into a subsequent single-player run.
pub fn cleanup_mp_loading(
    mut commands: Commands,
    mut config: ResMut<GameConfig>,
    backup: Option<Res<MpConfigBackup>>,
) {
    commands.remove_resource::<MpSpawnQueue>();
    commands.remove_resource::<MpLoadingSync>();

    if let Some(backup) = backup {
        config.seed = backup.previous_seed;
        config.current_level = backup.previous_current_level;
        commands.remove_resource::<MpConfigBackup>();
    }
    // The terrain was already enqueued (the queue holds owned clones of each
    // `Saved*` element), so clearing here doesn't affect the in-flight spawn.
    config.saved_flora.clear();
    config.saved_trees.clear();
    config.saved_ponds.clear();
    config.saved_bushes.clear();
    config.saved_boulders.clear();
}

/// Sets up the camera for the multiplayer game based on role.
///
/// Host uses the standard camera position; guest gets a mirrored view.
pub fn setup_mp_camera(
    session: Res<MultiplayerSession>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    if let Ok(mut transform) = camera_query.single_mut()
        && session.role == PeerRole::Guest
    {
        // Mirrored camera: opposite corner looking at origin
        *transform = Transform::from_xyz(1000.0, 2500.0, -2500.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
    }
}

/// Restores the default camera when leaving multiplayer.
pub fn restore_camera(mut camera_query: Query<&mut Transform, With<Camera3d>>) {
    if let Ok(mut transform) = camera_query.single_mut() {
        *transform = Transform::from_xyz(-1000.0, 2500.0, 2500.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
    }
}
