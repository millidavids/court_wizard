//! Core spawn-queue processing system (per-frame batched entity spawn).

use std::collections::HashSet;

use bevy::prelude::*;

use crate::config::GameConfig;
use crate::game::battlefield::components::BattlefieldAssets;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::loading::constants::{MAX_COMMANDER_ARCHERS, MAX_COMMANDER_INFANTRY};
use crate::game::loading::spawn_queue::{SpawnQueue, SpawnTask};
use crate::game::loading::upgrade_selection;
use crate::game::loading::upgrade_systems;
use crate::game::resources::CurrentLevel;
use crate::game::seeded_rng::resources::GameRng;
use crate::game::units::aerialist::AerialistAssets;
use crate::game::units::aerialist::systems as aerialist_systems;
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
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
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
        Res<crate::game::units::teleporter::resources::TeleporterAssets>,
    ),
    aerialist_assets: Res<AerialistAssets>,
    king_assets: Res<crate::game::units::king::resources::KingAssets>,
    boss_assets: (
        Res<crate::game::units::boss::ogre::resources::OgreAssets>,
        Res<crate::game::units::boss::hags::resources::HagAssets>,
        Res<crate::game::units::boss::dark_mage::resources::DarkMageAssets>,
        Res<crate::game::units::boss::ray::resources::RayAssets>,
    ),
    current_level: Res<CurrentLevel>,
    asset_stores: (
        ResMut<Assets<Mesh>>,
        ResMut<Assets<StandardMaterial>>,
        ResMut<Assets<crate::game::battlefield::ground_material::GroundMaterial>>,
        ResMut<Assets<crate::game::battlefield::ground_material::StoneNoiseMaterial>>,
    ),
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
        // Co-op host: drives the "both peers loaded" handshake with the guest
        // before entering the match (NetworkConnection is inert in SP/versus;
        // CoopLoadingSync is absent unless this is a co-op host load). Bundled
        // here to stay within Bevy's system-parameter count limit.
        ResMut<crate::networking::resources::NetworkConnection>,
        Option<ResMut<crate::game::multiplayer::coop::CoopLoadingSync>>,
    ),
    mut game_rng: ResMut<GameRng>,
) {
    let (
        infantry_assets,
        archer_assets,
        assassin_assets,
        dispeller_assets,
        shielder_assets,
        healer_assets,
        teleporter_assets,
    ) = &unit_assets;
    let (ogre_assets, hag_assets, dark_mage_assets, ray_assets) = &boss_assets;
    let (mut meshes, mut materials, mut ground_materials, mut stone_materials) = asset_stores;
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
    let (mut channel_change, mut obstacle_events, mut connection, coop_sync) = message_writers;
    // `CoopLoadingSync` is present only while a co-op HOST is loading on this SP
    // path — use it to place the host's cauldron at the shared between-wizards spot.
    let coop_host_load = coop_sync.is_some();

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
                    &mut game_rng.0,
                    &mut commands,
                    infantry_assets,
                    &mut materials,
                    unit_index,
                );
            }
            SpawnTask::AttackerInfantry { unit_index, level } => {
                infantry_systems::spawn_single_attacker(
                    &mut game_rng.0,
                    &mut commands,
                    infantry_assets,
                    &mut materials,
                    unit_index,
                    level,
                );
            }
            SpawnTask::DefenderArcher { unit_index } => {
                archer_systems::spawn_single_defender_archer(
                    &mut game_rng.0,
                    &mut commands,
                    archer_assets,
                    &mut materials,
                    unit_index,
                );
            }
            SpawnTask::AttackerArcher { unit_index, level } => {
                archer_systems::spawn_single_attacker_archer(
                    &mut game_rng.0,
                    &mut commands,
                    archer_assets,
                    &mut materials,
                    unit_index,
                    level,
                );
            }
            SpawnTask::AttackerAssassin { unit_index, level } => {
                assassin_systems::spawn_single_attacker_assassin(
                    &mut game_rng.0,
                    &mut commands,
                    assassin_assets,
                    &mut materials,
                    unit_index,
                    level,
                );
            }
            SpawnTask::AttackerAerialist { unit_index, level } => {
                aerialist_systems::spawn_single_attacker_aerialist(
                    &mut game_rng.0,
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
                    spell_visual_assets,
                    &mut king_spawned,
                );
            }
            SpawnTask::KingsGuard { guard_index } => {
                infantry_systems::spawn_single_kings_guard(
                    &mut game_rng.0,
                    &mut commands,
                    infantry_assets,
                    &mut materials,
                    guard_index,
                );
            }
            SpawnTask::Brute => {
                crate::game::units::brute::systems::spawn_brute(
                    &mut game_rng.0,
                    commands.reborrow(),
                    Res::clone(infantry_assets),
                    &mut materials,
                    Res::clone(&current_level),
                );
            }
            SpawnTask::Teleporter => {
                crate::game::units::teleporter::systems::spawn_single_teleporter(
                    &mut game_rng.0,
                    &mut commands,
                    teleporter_assets,
                    &mut materials,
                );
            }
            SpawnTask::Ogre => {
                crate::game::units::boss::ogre::systems::spawn_ogre(
                    &mut game_rng.0,
                    commands.reborrow(),
                    Res::clone(ogre_assets),
                    &mut materials,
                );
            }
            SpawnTask::Hags => {
                crate::game::units::boss::hags::systems::spawn_hags(
                    &mut game_rng.0,
                    commands.reborrow(),
                    Res::clone(hag_assets),
                );
            }
            SpawnTask::DarkMage => {
                crate::game::units::boss::dark_mage::systems::spawn_dark_mage(
                    &mut game_rng.0,
                    commands.reborrow(),
                    Res::clone(dark_mage_assets),
                );
            }
            SpawnTask::Ray => {
                crate::game::units::boss::ray::systems::spawn_ray(
                    &mut game_rng.0,
                    commands.reborrow(),
                    Res::clone(ray_assets),
                );
            }
            SpawnTask::Battlefield => {
                crate::game::battlefield::systems::setup_battlefield(
                    &mut game_rng.0,
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut ground_materials,
                    &mut stone_materials,
                    battlefield_assets,
                    Transform::IDENTITY,
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
            SpawnTask::CoopGuestWizard { guest_wizard } => {
                // Co-op: the guest's wizard proxy, beside the host on the shared
                // battlefield. `role = Host, is_host_wizard = false` inserts the
                // `GuestWizard` marker so the host processes the guest's spell
                // commands against it.
                if let Some(assets) = wizard_assets_opt {
                    crate::game::multiplayer::spawning::spawn_mp_wizard(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        crate::game::constants::WIZARD_COOP_POSITION,
                        guest_wizard,
                        crate::networking::resources::PeerRole::Host,
                        false,
                        true, // co-op proxy → SP range
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
                    // Co-op host: shared cauldron between the two wizards (same spot
                    // the guest spawns it). Single-player: beside the lone wizard.
                    let cauldron_pos = if coop_host_load {
                        crate::game::cauldron::constants::CAULDRON_COOP_POSITION
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
                            spell_visual_assets,
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
                    tree.sprite_index,
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

    // Transition to InGame when all tasks are complete.
    if spawn_queue.is_complete() {
        if let Some(mut sync) = coop_sync {
            // Co-op host: exchange `GameLoaded` with the guest (loading in
            // MultiplayerLoading) before entering the match, so neither peer
            // starts simulating/ghosting before both worlds exist.
            use crate::networking::protocol::NetworkMessage;
            use crate::networking::resources::ConnectionState;
            if !sync.my_loaded {
                sync.my_loaded = true;
                connection
                    .outgoing_messages
                    .push(NetworkMessage::GameLoaded);
            }
            if connection
                .incoming_messages
                .iter()
                .any(|m| matches!(m, NetworkMessage::GameLoaded))
            {
                connection
                    .incoming_messages
                    .retain(|m| !matches!(m, NetworkMessage::GameLoaded));
                sync.peer_loaded = true;
            }
            // Enter the match once both peers have loaded AND the link is live.
            let both_ready = sync.peer_loaded && connection.state == ConnectionState::Connected;
            // …but if the guest drops during the handshake (Failed/Disconnected),
            // DON'T hang forever waiting for a `GameLoaded` that will never come —
            // the co-op design is "guest drops → host continues solo", and the
            // host's SP-shell world is already fully built. Proceed into InGame
            // alone; `init_coop_host` reads the dead connection and leaves
            // `CoopGuestConnected` false (no +30% buff), and the guest can rejoin
            // at the next level boundary. (`detect_mp_loading_disconnect` only
            // covers `MultiplayerLoading`, which the co-op host never enters.)
            let guest_gone = matches!(
                connection.state,
                ConnectionState::Failed | ConnectionState::Disconnected
            );
            if sync.my_loaded && (both_ready || guest_gone) {
                commands.remove_resource::<crate::game::multiplayer::coop::CoopLoadingSync>();
                channel_change.write(ChannelChangeMessage);
                next_state.set(AppState::InGame);
            }
        } else {
            channel_change.write(ChannelChangeMessage);
            next_state.set(AppState::InGame);
        }
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
