//! Multiplayer match lifecycle: run conditions, resource init/cleanup, and
//! the shared `do_mp_disconnect` helper used by score, pause, and disconnected screens.

use bevy::prelude::*;

use crate::config::{GameConfig, WizardType};
use crate::game::attack_cycle::GlobalAttackCycle;
use crate::game::cauldron::resources::CauldronBuffs;
use crate::game::resources::{GameOutcome, KillStats};
use crate::game::units::infantry::components::DefendersActivated;
use crate::networking::entity_map::{EntityIdCounter, NetworkEntityMap};
use crate::networking::resources::NetworkConnection;
use crate::networking::session::MultiplayerSession;
use crate::networking::snapshot::SnapshotTick;
use crate::networking::transport::TransportHandle;
use crate::state::{AppState, MultiplayerGameState};
use crate::ui::wizard_tower::{CoopHostSelection, MultiplayerLobby};

use super::super::session_reset::reset_multiplayer_to_baseline;

use super::super::components::{OnMultiplayerGameScreen, PendingRematch};

/// Safe run condition for `MultiplayerGameState` sub-states.
///
/// Returns true for both Running and Paused states, since the escape menu
/// overlay does not pause gameplay. Unlike `in_state(MultiplayerGameState::*)`,
/// these won't panic if the sub-state resource has already been removed.
pub(crate) fn in_mp_running(state: Option<Res<State<MultiplayerGameState>>>) -> bool {
    state.is_some_and(|s| {
        matches!(
            *s.get(),
            MultiplayerGameState::Running
                | MultiplayerGameState::Paused
                | MultiplayerGameState::SpellBook
                | MultiplayerGameState::CauldronMenu
        )
    })
}

pub(crate) fn in_mp_score_screen(state: Option<Res<State<MultiplayerGameState>>>) -> bool {
    state.is_some_and(|s| *s.get() == MultiplayerGameState::ScoreScreen)
}

pub(crate) fn in_mp_paused(state: Option<Res<State<MultiplayerGameState>>>) -> bool {
    state.is_some_and(|s| *s.get() == MultiplayerGameState::Paused)
}

pub(crate) fn in_mp_disconnected(state: Option<Res<State<MultiplayerGameState>>>) -> bool {
    state.is_some_and(|s| *s.get() == MultiplayerGameState::Disconnected)
}

/// Initializes resources needed for multiplayer gameplay.
///
/// GlobalAttackCycle, KillStats, DefendersActivated, and GameOutcome are already
/// initialized by GamePlugin at startup. We reset them here to ensure clean state
/// and additionally insert MP-only resources.
#[allow(clippy::too_many_arguments)]
pub(crate) fn init_mp_game(
    mut commands: Commands,
    mut attack_cycle: ResMut<GlobalAttackCycle>,
    mut kill_stats: ResMut<KillStats>,
    mut defenders_activated: ResMut<DefendersActivated>,
    mut game_outcome: ResMut<GameOutcome>,
    connection: Res<NetworkConnection>,
    config: Res<GameConfig>,
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
    arcanorouter_state: Option<
        Res<crate::game::units::wizard::archetypes::arcanorouter::ArcanoRouterState>,
    >,
) {
    // Reset globally-owned resources to clean state for this match
    *attack_cycle = GlobalAttackCycle::default();
    kill_stats.reset();
    defenders_activated.active = true;
    *game_outcome = GameOutcome::Victory;

    // Insert PeerId based on role (host=0, guest=1)
    use crate::networking::crdt::PeerId;
    use crate::networking::resources::PeerRole;
    let peer_id = match connection.role {
        Some(PeerRole::Host) => PeerId(PeerId::HOST),
        _ => PeerId(PeerId::GUEST),
    };
    commands.insert_resource(peer_id);

    // Point the spell-origin resource at the local player's wizard so all the
    // shared spell-casting code spawns visuals at the correct corner. The co-op
    // guest stands beside the host on the SP battlefield (`SPELL_COOP_ORIGIN`),
    // whereas the versus guest is mirrored to the opposite corner
    // (`SPELL_2_ORIGIN`). Only the GUEST runs `init_mp_game`; the host keeps the
    // default `SPELL_ORIGIN` from single-player startup.
    use crate::game::constants::{SPELL_2_ORIGIN, SPELL_COOP_ORIGIN, SPELL_ORIGIN};
    use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
    let is_coop = session.as_deref().is_some_and(|s| s.is_coop());
    let origin = match connection.role {
        Some(PeerRole::Guest) if is_coop => SPELL_COOP_ORIGIN,
        Some(PeerRole::Guest) => SPELL_2_ORIGIN,
        _ => SPELL_ORIGIN,
    };
    commands.insert_resource(LocalSpellOrigin(origin));

    // Insert MP-only resources
    commands.init_resource::<EntityIdCounter>();
    commands.init_resource::<NetworkEntityMap>();
    commands.init_resource::<SnapshotTick>();
    commands.init_resource::<crate::networking::snapshot::SpellSnapshotData>();
    commands.init_resource::<super::super::components::SpellEffectEntityMap>();
    commands.init_resource::<super::super::spell_sync::LatestSpellSnapshot>();
    // Per-match wizard spell-stat accumulator for the score screen.
    commands.init_resource::<super::super::score_stats::LocalWizardStats>();

    // Pin the Arcanorouter's range allocation for the setup stage (per-peer:
    // `config.wizard_type` was just synced to THIS peer's wizard).
    if config.wizard_type == WizardType::Arcanorouter {
        let baseline = arcanorouter_state
            .map(|s| s.range_allocation)
            .unwrap_or(100.0);
        commands.insert_resource(
            crate::game::units::wizard::archetypes::arcanorouter::ArcanoRouterSetupBaseline(
                baseline,
            ),
        );
    }

    // SpellVisualAssets is initialized globally at startup, no MP-specific asset init needed.
}

/// Backs up the single-player `GameConfig.wizard_type` for the lifetime of a
/// multiplayer match so `cleanup_mp_game` can restore it on exit (a later
/// single-player run must not inherit the MP archetype).
#[derive(Resource)]
pub(crate) struct MpWizardTypeBackup(pub WizardType);

/// Points `GameConfig.wizard_type` at THIS peer's wizard for the duration of the
/// match.
///
/// Every archetype run-condition (`is_warglock`, `is_meteorologist`,
/// `is_swordcerer`, …) reads `GameConfig.wizard_type`. In multiplayer that field
/// is otherwise whatever the last single-player run left behind, so without this
/// the local archetype's systems silently gate against the wrong wizard. We sync
/// it from the authoritative `MultiplayerSession` on match entry and stash the
/// previous value for restoration on exit.
///
/// Runs on `OnEnter(AppState::MultiplayerGame)` before `init_mp_game`. Other
/// `OnEnter(AppState::MultiplayerGame)` systems gated on an archetype
/// run-condition must be ordered `.after(sync_wizard_type_from_session)`.
pub(crate) fn sync_wizard_type_from_session(
    mut commands: Commands,
    session: Res<MultiplayerSession>,
    backup: Option<Res<MpWizardTypeBackup>>,
    mut config: ResMut<GameConfig>,
) {
    // Stash the single-player wizard type ONCE — don't clobber it if a rematch
    // re-enters and re-syncs (the stash already holds the true SP value).
    if backup.is_none() {
        commands.insert_resource(MpWizardTypeBackup(config.wizard_type));
    }
    config.wizard_type = session.local_wizard();
}

/// Cleans up multiplayer game entities and resources.
///
/// If `PendingRematch` is present, the `MultiplayerSession` is kept alive
/// so the WebRTC connection persists through the rematch flow.
#[allow(clippy::too_many_arguments)]
pub(crate) fn cleanup_mp_game(
    mut commands: Commands,
    mp_entities: Query<Entity, With<OnMultiplayerGameScreen>>,
    pending_rematch: Option<Res<PendingRematch>>,
    mut attack_cycle: ResMut<GlobalAttackCycle>,
    mut kill_stats: ResMut<KillStats>,
    mut defenders_activated: ResMut<DefendersActivated>,
    mut cauldron_buffs: ResMut<CauldronBuffs>,
    mut game_outcome: ResMut<GameOutcome>,
    mut config: ResMut<GameConfig>,
    wizard_type_backup: Option<Res<MpWizardTypeBackup>>,
) {
    // `OnGameplayScreen` entities (battlefield + terrain, spawned via the
    // reused single-player path) are despawned by `shared_systems::cleanup_game`,
    // which is already registered on OnExit(MultiplayerGame). We only handle
    // the multiplayer-specific marker here.
    for entity in &mp_entities {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.try_despawn();
        }
    }

    // Reset globally-owned resources to defaults rather than removing them,
    // since other plugins (GamePlugin, InfantryPlugin, CauldronPlugin) initialise
    // these at startup and their run conditions expect them to always exist.
    *attack_cycle = GlobalAttackCycle::default();
    kill_stats.reset();
    defenders_activated.active = false;
    // Clear the setup-stage immunity flag so it never leaks into a later
    // single-player game (e.g. if the match ended mid-setup).
    crate::game::units::components::set_setup_immunity(false);
    commands
        .remove_resource::<crate::game::units::wizard::archetypes::arcanorouter::ArcanoRouterSetupBaseline>();
    cauldron_buffs.active_buffs.clear();
    *game_outcome = GameOutcome::Victory;

    // Restore the spell origin to the single-player default so a subsequent
    // SP run doesn't inherit the guest's mirrored origin.
    commands
        .insert_resource(crate::game::units::wizard::spells::utils::LocalSpellOrigin::default());

    // Restore the single-player wizard type stashed at match entry
    // (`sync_wizard_type_from_session`) so a later SP run doesn't inherit the MP
    // archetype. On rematch the next entry re-syncs from the session.
    if let Some(backup) = wizard_type_backup {
        config.wizard_type = backup.0;
        commands.remove_resource::<MpWizardTypeBackup>();
    }

    // Remove MP-only resources that are created in init_mp_game.
    commands.remove_resource::<crate::networking::crdt::PeerId>();
    commands.remove_resource::<EntityIdCounter>();
    commands.remove_resource::<NetworkEntityMap>();
    commands.remove_resource::<SnapshotTick>();
    commands.remove_resource::<super::super::components::SpellEffectEntityMap>();
    commands.remove_resource::<crate::networking::snapshot::SpellSnapshotData>();
    commands.remove_resource::<super::super::spell_sync::LatestSpellSnapshot>();
    commands.remove_resource::<super::super::score_stats::LocalWizardStats>();

    // Tear down the pathfinding grid that MP loading populated with this
    // match's terrain (boulders, ponds, etc.). Without removal, the grid
    // leaks into MetaGame; a subsequent SP run reinitialises it but the
    // stale ~MB-scale resource sits in memory in the meantime.
    // `detect_mp_loading_disconnect` already removes this on the abort
    // path; this is the corresponding teardown for a clean match exit.
    commands.remove_resource::<crate::game::pathfinding::resources::PathfindingGrid>();

    // Only remove the session if this is NOT a rematch — keep connection alive for rematch
    if pending_rematch.is_none() {
        commands.remove_resource::<MultiplayerSession>();
    }
}

/// Tears down the active multiplayer session and returns to the main menu.
///
/// Shared by the score-screen, pause-menu, and disconnected-overlay disconnect
/// paths plus the score-screen Escape handler. `transport` is `Option`: the
/// disconnected-overlay path passes `None` (the peer is already gone, so there's
/// nothing to signal); every other path passes the live handle so the peer is
/// told to disconnect.
///
/// This is just the canonical baseline reset plus the navigation. Note the reset
/// also clears `PendingRematch`: disconnecting cancels any rematch, and clearing
/// it defensively stops a confirm-rematch-on-the-same-frame-as-leave race from
/// carrying it into the main menu and rematching on the session we just tore
/// down. (`PendingRematch` is only set on the normal both-ready path, which never
/// calls this helper, so a legitimate rematch is unaffected.)
#[allow(clippy::too_many_arguments)]
pub(crate) fn do_mp_disconnect(
    connection: &mut NetworkConnection,
    transport: Option<&TransportHandle>,
    steam_client: Option<&bevy_steamworks::Client>,
    steam_lobby: Option<&mut crate::steam::multiplayer::SteamLobbyState>,
    steam_socket: Option<&mut crate::steam::multiplayer::SteamP2pSocket>,
    lobby: &mut MultiplayerLobby,
    host_selection: &mut CoopHostSelection,
    commands: &mut Commands,
    next_app_state: &mut NextState<AppState>,
    session_present: bool,
) {
    reset_multiplayer_to_baseline(
        "multiplayer disconnect",
        commands,
        connection,
        lobby,
        host_selection,
        transport,
        steam_client,
        steam_lobby,
        steam_socket,
        session_present,
    );
    next_app_state.set(AppState::MainMenu);
}
