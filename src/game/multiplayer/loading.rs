//! Multiplayer loading systems.
//!
//! Builds and processes a multiplayer-specific spawn queue that sets up
//! dual castles, armies for both sides, and two wizards. The host spawns
//! all gameplay entities; the guest spawns only visual entities (units
//! will come from state snapshots in Milestone 4).

use bevy::prelude::*;

use super::spawning::*;
use crate::game::battlefield::components::BattlefieldAssets;
use crate::game::constants::*;
use crate::game::loading::resources::LoadingProgress;
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
    /// Spawn the battlefield ground plane and lighting.
    Battlefield,
    /// Spawn Castle 1 (host's castle).
    Castle1,
    /// Spawn Castle 2 (guest's castle).
    Castle2,
    /// Initialize the pathfinding grid.
    PathfindingGrid,
    /// Load wizard sprite sheet assets.
    LoadWizardAssets,
    /// Spawn host's wizard at Castle 1.
    HostWizard,
    /// Spawn guest's wizard at Castle 2.
    GuestWizard,
    /// Spawn a host-side defender infantry (near Castle 1).
    HostInfantry { unit_index: u32 },
    /// Spawn a host-side defender archer (near Castle 1).
    HostArcher { unit_index: u32 },
    /// Spawn a guest-side infantry (near Castle 2).
    GuestInfantry { unit_index: u32 },
    /// Spawn a guest-side archer (near Castle 2).
    GuestArcher { unit_index: u32 },
    /// Spawn the host-side King near Castle 1.
    HostKing,
    /// Spawn a host-side King's Guard unit.
    HostKingsGuard { guard_index: u32 },
    /// Spawn the guest-side King near Castle 2.
    GuestKing,
    /// Spawn a guest-side King's Guard unit.
    GuestKingsGuard { guard_index: u32 },
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

/// Initializes the multiplayer loading spawn queue.
pub fn init_mp_loading(mut commands: Commands, session: Res<MultiplayerSession>) {
    commands.insert_resource(LoadingProgress::new());
    commands.insert_resource(MpLoadingSync::default());

    let mut queue = MpSpawnQueue::new();

    // Both host and guest spawn the visual world
    queue.tasks.push(MpSpawnTask::Battlefield);
    queue.tasks.push(MpSpawnTask::Castle1);
    queue.tasks.push(MpSpawnTask::Castle2);
    queue.tasks.push(MpSpawnTask::PathfindingGrid);

    // Host spawns all gameplay entities (both armies)
    if session.role == PeerRole::Host {
        // Host's King and Guard
        queue.tasks.push(MpSpawnTask::HostKing);
        for i in 0..KINGS_GUARD_COUNT {
            queue
                .tasks
                .push(MpSpawnTask::HostKingsGuard { guard_index: i });
        }

        // Guest's King and Guard
        queue.tasks.push(MpSpawnTask::GuestKing);
        for i in 0..KINGS_GUARD_COUNT {
            queue
                .tasks
                .push(MpSpawnTask::GuestKingsGuard { guard_index: i });
        }

        // Host-side army (near Castle 1)
        for i in 0..MP_INFANTRY_COUNT {
            queue
                .tasks
                .push(MpSpawnTask::HostInfantry { unit_index: i });
        }
        for i in 0..MP_ARCHER_COUNT {
            queue.tasks.push(MpSpawnTask::HostArcher { unit_index: i });
        }

        // Guest-side army (near Castle 2)
        for i in 0..MP_INFANTRY_COUNT {
            queue
                .tasks
                .push(MpSpawnTask::GuestInfantry { unit_index: i });
        }
        for i in 0..MP_ARCHER_COUNT {
            queue.tasks.push(MpSpawnTask::GuestArcher { unit_index: i });
        }
    }

    // Load wizard sprite sheet, then spawn wizards
    queue.tasks.push(MpSpawnTask::LoadWizardAssets);
    queue.tasks.push(MpSpawnTask::HostWizard);
    queue.tasks.push(MpSpawnTask::GuestWizard);

    commands.insert_resource(queue);
}

/// Processes one multiplayer spawn task per frame.
#[allow(clippy::too_many_arguments)]
pub fn process_mp_spawn_queue(
    mut commands: Commands,
    mut loading_progress: ResMut<LoadingProgress>,
    mut spawn_queue: ResMut<MpSpawnQueue>,
    mut loading_sync: ResMut<MpLoadingSync>,
    mut connection: ResMut<NetworkConnection>,
    session: Res<MultiplayerSession>,
    infantry_assets: Res<InfantryAssets>,
    archer_assets: Res<ArcherAssets>,
    king_assets: Res<crate::game::units::king::resources::KingAssets>,
    mut king_spawned: ResMut<KingSpawned>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<AppState>>,
    battlefield_assets: Res<BattlefieldAssets>,
    asset_server: Res<AssetServer>,
    wizard_assets: Option<Res<WizardAssets>>,
) {
    // Process one task per frame
    if let Some(task) = spawn_queue.pop_next() {
        match task {
            MpSpawnTask::Battlefield => {
                spawn_mp_battlefield(&mut commands, &mut meshes, &mut materials);
            }
            MpSpawnTask::Castle1 => {
                spawn_castle(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &battlefield_assets,
                    CASTLE_POSITION,
                    CASTLE_ROTATION_DEGREES,
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
                if let Some(ref assets) = wizard_assets {
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
            }
            MpSpawnTask::GuestWizard => {
                if let Some(ref assets) = wizard_assets {
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
            }
            MpSpawnTask::HostInfantry { unit_index } => {
                spawn_mp_infantry(
                    &mut commands,
                    &infantry_assets,
                    &mut materials,
                    unit_index,
                    Team::Defenders,
                    true, // host side (Castle 1)
                );
            }
            MpSpawnTask::GuestInfantry { unit_index } => {
                spawn_mp_infantry(
                    &mut commands,
                    &infantry_assets,
                    &mut materials,
                    unit_index,
                    Team::Attackers,
                    false, // guest side (Castle 2)
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
                    &mut meshes,
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
                    &mut meshes,
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
        }
    }

    loading_progress.advance();

    // When queue is done and minimum frames reached, signal loaded
    if spawn_queue.is_complete() && loading_progress.is_complete() && !loading_sync.my_loaded {
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

    // Both loaded — transition to gameplay
    if loading_sync.my_loaded && loading_sync.peer_loaded {
        next_state.set(AppState::MultiplayerGame);
    }
}

/// Cleans up multiplayer loading resources.
pub fn cleanup_mp_loading(mut commands: Commands) {
    commands.remove_resource::<LoadingProgress>();
    commands.remove_resource::<MpSpawnQueue>();
    commands.remove_resource::<MpLoadingSync>();
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
