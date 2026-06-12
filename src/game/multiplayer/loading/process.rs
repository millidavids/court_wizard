//! Multiplayer spawn-queue processor and peer-sync handshake.

use bevy::prelude::*;

use super::queue::{MpSpawnQueue, MpSpawnTask};
use super::resources::MpLoadingSync;
use crate::game::battlefield::components::BattlefieldAssets;
use crate::game::battlefield::ground_material::{GroundMaterial, StoneNoiseMaterial};
use crate::game::constants::*;
use crate::game::multiplayer::spawning::*;
use crate::game::pathfinding::messages::ObstacleChanged;
use crate::game::seeded_rng::resources::GameRng;
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
use crate::networking::resources::{ConnectionState, NetworkConnection, PeerRole};
use crate::networking::session::MultiplayerSession;
use crate::state::AppState;

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
    // coords. The VERSUS guest spawns its copy rotated 180° around the world
    // origin so that, combined with its mirrored camera, the asymmetric SP wall
    // art (right wall with tunnels, left wall) appears correctly oriented from
    // the guest's perspective. The CO-OP guest keeps the single-player camera
    // (no mirror — see `setup_mp_camera`), so it must NOT rotate the world
    // either, or the wall art would appear backwards.
    let origin_transform = if session.role == PeerRole::Guest && !session.is_coop() {
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
                    session.is_coop(),
                    assets,
                );
            }
            MpSpawnTask::GuestWizard => {
                let Some(ref assets) = wizard_assets else {
                    spawn_queue.tasks.insert(0, MpSpawnTask::GuestWizard);
                    break;
                };
                // Co-op: the guest stands beside the host (`WIZARD_COOP_POSITION`,
                // same camera). Versus: the mirrored opposite corner.
                let guest_pos = if session.is_coop() {
                    WIZARD_COOP_POSITION
                } else {
                    WIZARD_2_POSITION
                };
                spawn_mp_wizard(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    guest_pos,
                    session.guest_wizard,
                    session.role,
                    false,
                    session.is_coop(),
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
                // Place the cauldron beside the LOCAL wizard. Versus guest: the
                // mirrored corner (`CAULDRON_2_POSITION`). Co-op guest: beside the
                // host (`CAULDRON_COOP_POSITION`). Host (versus): `CAULDRON_POSITION`.
                let cauldron_pos = if session.role == PeerRole::Guest {
                    if session.is_coop() {
                        crate::game::cauldron::constants::CAULDRON_COOP_POSITION
                    } else {
                        crate::game::cauldron::constants::CAULDRON_2_POSITION
                    }
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
        && connection.state == ConnectionState::Connected
    {
        next_state.set(AppState::MultiplayerGame);
    }
}
