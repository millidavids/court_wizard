//! Multiplayer game systems (init, cleanup, score screen, pause, disconnect).

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::config::{GameConfig, WizardType};
use crate::game::cauldron::resources::CauldronBuffs;
use crate::game::input::messages::MouseClicked;
use crate::game::plugin::GlobalAttackCycle;
use crate::game::resources::{GameOutcome, KillStats};
use crate::game::units::infantry::components::DefendersActivated;
use crate::networking::entity_map::{EntityIdCounter, NetworkEntityMap};
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{ConnectionState, NetworkConnection};
use crate::networking::session::MultiplayerSession;
use crate::networking::snapshot::SnapshotTick;
use crate::networking::transport::{TransportCommand, TransportHandle};
use crate::state::{AppState, MultiplayerGameState};
use crate::ui::components::ButtonStyle;
use crate::ui::constants::{BUTTON_BG, BUTTON_BORDER, TEXT_PRIMARY};
use crate::ui::systems::spawn_button;
use crate::ui::wizard_tower::MultiplayerLobby;

use super::components::{
    MpDisconnectedButtonAction, MpPauseButtonAction, MpRematchState, MpScoreButtonAction,
    MpStatValueText, OnMpDisconnectedScreen, OnMpPauseScreen, OnMpScoreScreen,
    OnMultiplayerGameScreen, PendingRematch, RematchStatusText,
};

/// Safe run condition for `MultiplayerGameState` sub-states.
///
/// Returns true for both Running and Paused states, since the escape menu
/// overlay does not pause gameplay. Unlike `in_state(MultiplayerGameState::*)`,
/// these won't panic if the sub-state resource has already been removed.
pub(super) fn in_mp_running(state: Option<Res<State<MultiplayerGameState>>>) -> bool {
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

pub(super) fn in_mp_score_screen(state: Option<Res<State<MultiplayerGameState>>>) -> bool {
    state.is_some_and(|s| *s.get() == MultiplayerGameState::ScoreScreen)
}

pub(super) fn in_mp_paused(state: Option<Res<State<MultiplayerGameState>>>) -> bool {
    state.is_some_and(|s| *s.get() == MultiplayerGameState::Paused)
}

pub(super) fn in_mp_disconnected(state: Option<Res<State<MultiplayerGameState>>>) -> bool {
    state.is_some_and(|s| *s.get() == MultiplayerGameState::Disconnected)
}

/// Initializes resources needed for multiplayer gameplay.
///
/// GlobalAttackCycle, KillStats, DefendersActivated, and GameOutcome are already
/// initialized by GamePlugin at startup. We reset them here to ensure clean state
/// and additionally insert MP-only resources.
pub(super) fn init_mp_game(
    mut commands: Commands,
    mut attack_cycle: ResMut<GlobalAttackCycle>,
    mut kill_stats: ResMut<KillStats>,
    mut defenders_activated: ResMut<DefendersActivated>,
    mut game_outcome: ResMut<GameOutcome>,
    connection: Res<NetworkConnection>,
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
    // shared spell-casting code spawns visuals at the correct corner.
    use crate::game::constants::{SPELL_2_ORIGIN, SPELL_ORIGIN};
    use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
    let origin = match connection.role {
        Some(PeerRole::Guest) => SPELL_2_ORIGIN,
        _ => SPELL_ORIGIN,
    };
    commands.insert_resource(LocalSpellOrigin(origin));

    // Insert MP-only resources
    commands.init_resource::<EntityIdCounter>();
    commands.init_resource::<NetworkEntityMap>();
    commands.init_resource::<SnapshotTick>();
    commands.init_resource::<crate::networking::snapshot::SpellSnapshotData>();
    commands.init_resource::<super::components::SpellEffectEntityMap>();
    commands.init_resource::<super::spell_sync::LatestSpellSnapshot>();
    // Per-match wizard spell-stat accumulator for the score screen.
    commands.init_resource::<super::score_stats::LocalWizardStats>();

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
pub(super) fn cleanup_mp_game(
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
    commands.remove_resource::<super::components::SpellEffectEntityMap>();
    commands.remove_resource::<crate::networking::snapshot::SpellSnapshotData>();
    commands.remove_resource::<super::spell_sync::LatestSpellSnapshot>();
    commands.remove_resource::<super::score_stats::LocalWizardStats>();

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

// ── Score Screen Constants ────────────────────────────────────────────

const SCORE_BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.85);
const SCORE_TITLE_COLOR: Color = Color::srgb(0.95, 0.95, 0.95);
const SCORE_TEXT_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);

const SCORE_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 250.0,
    height: 65.0,
    border_width: 3.0,
    font_size: 20.0,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};

// ── Score Screen Systems ──────────────────────────────────────────────

/// The four scoreboard rows, as `(label, your-side marker, enemy-side marker)`.
const STAT_ROWS: [(&str, MpStatValueText, MpStatValueText); 4] = [
    (
        "Kills",
        MpStatValueText::YourKills,
        MpStatValueText::EnemyKills,
    ),
    (
        "Deaths",
        MpStatValueText::YourDeaths,
        MpStatValueText::EnemyDeaths,
    ),
    (
        "Damage",
        MpStatValueText::YourDamage,
        MpStatValueText::EnemyDamage,
    ),
    (
        "Healed",
        MpStatValueText::YourHealing,
        MpStatValueText::EnemyHealing,
    ),
];

/// Formats a single `MatchStats` field for display. Returns "0" when stats
/// aren't available yet (e.g. the host's enemy column before the guest's
/// `WizardStatsReport` arrives). Damage/healing are shown as whole numbers.
fn stat_value_text(
    stats: Option<&super::score_stats::MatchStats>,
    which: MpStatValueText,
) -> String {
    let Some(s) = stats else {
        return "0".to_string();
    };
    match which {
        MpStatValueText::YourKills => s.your_kills.to_string(),
        MpStatValueText::YourDeaths => s.your_deaths.to_string(),
        MpStatValueText::YourDamage => (s.your_damage.round() as u32).to_string(),
        MpStatValueText::YourHealing => (s.your_healing.round() as u32).to_string(),
        MpStatValueText::EnemyKills => s.enemy_kills.to_string(),
        MpStatValueText::EnemyDeaths => s.enemy_deaths.to_string(),
        MpStatValueText::EnemyDamage => (s.enemy_damage.round() as u32).to_string(),
        MpStatValueText::EnemyHealing => (s.enemy_healing.round() as u32).to_string(),
    }
}

/// Spawns one label+value stat row; the value `Text` is tagged with `marker`
/// so `update_mp_stat_values` can refresh it reactively.
fn spawn_stat_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    marker: MpStatValueText,
    stats: Option<&super::score_stats::MatchStats>,
) {
    parent
        .spawn(Node {
            width: Val::Px(200.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: Val::Px(24.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont::from_font_size(20.0),
                TextColor(SCORE_TEXT_COLOR),
            ));
            row.spawn((
                marker,
                Text::new(stat_value_text(stats, marker)),
                TextFont::from_font_size(20.0),
                TextColor(SCORE_TITLE_COLOR),
            ));
        });
}

/// Spawns one stat column (header + the four rows). `is_enemy` selects the
/// enemy-side markers so the value nodes update from the correct `MatchStats`
/// fields.
fn spawn_stat_column(
    parent: &mut ChildSpawnerCommands,
    header: &str,
    is_enemy: bool,
    stats: Option<&super::score_stats::MatchStats>,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|col| {
            col.spawn((
                Text::new(header),
                TextFont::from_font_size(26.0),
                TextColor(SCORE_TITLE_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));
            for (label, your_marker, enemy_marker) in STAT_ROWS {
                let marker = if is_enemy { enemy_marker } else { your_marker };
                spawn_stat_row(col, label, marker, stats);
            }
        });
}

/// Spawns the multiplayer score screen UI: a title over three columns —
/// enemy stats, your stats, and the Rematch/Disconnect buttons.
pub(super) fn setup_mp_score_screen(
    mut commands: Commands,
    game_outcome: Res<GameOutcome>,
    match_stats: Option<Res<super::score_stats::MatchStats>>,
) {
    commands.init_resource::<MpRematchState>();

    let title_text = match *game_outcome {
        GameOutcome::Victory => "VICTORY",
        _ => "DEFEAT",
    };
    let stats = match_stats.as_deref();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(24.0),
                ..default()
            },
            BackgroundColor(SCORE_BG_COLOR),
            OnMpScoreScreen,
            OnMultiplayerGameScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new(title_text),
                TextFont::from_font_size(60.0),
                TextColor(SCORE_TITLE_COLOR),
            ));

            // Subtitle for King death
            if *game_outcome == GameOutcome::DefeatKingDied {
                parent.spawn((
                    Text::new("Your King was slain!"),
                    TextFont::from_font_size(24.0),
                    TextColor(SCORE_TEXT_COLOR),
                ));
            }

            // Three columns: enemy stats | your stats | buttons
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexStart,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(60.0),
                    margin: UiRect::top(Val::Px(16.0)),
                    ..default()
                })
                .with_children(|columns| {
                    spawn_stat_column(columns, "ENEMY", true, stats);
                    spawn_stat_column(columns, "YOU", false, stats);

                    // Right column: buttons + rematch status text
                    columns
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(15.0),
                            ..default()
                        })
                        .with_children(|buttons| {
                            spawn_button(
                                buttons,
                                "Rematch",
                                MpScoreButtonAction::Rematch,
                                &SCORE_BUTTON_STYLE,
                            );
                            spawn_button(
                                buttons,
                                "Disconnect",
                                MpScoreButtonAction::Disconnect,
                                &SCORE_BUTTON_STYLE,
                            );
                            buttons.spawn((
                                RematchStatusText,
                                Text::new(""),
                                TextFont::from_font_size(18.0),
                                TextColor(SCORE_TEXT_COLOR),
                            ));
                        });
                });
        });
}

/// Refreshes the score-screen stat value nodes whenever `MatchStats` changes —
/// notably when the guest's `WizardStatsReport` fills in the host's enemy column.
pub(super) fn update_mp_stat_values(
    match_stats: Res<super::score_stats::MatchStats>,
    mut values: Query<(&MpStatValueText, &mut Text)>,
) {
    for (marker, mut text) in &mut values {
        **text = stat_value_text(Some(&match_stats), *marker);
    }
}

/// Cleans up score screen entities and resources.
pub(super) fn cleanup_mp_score_screen(
    mut commands: Commands,
    score_entities: Query<Entity, With<OnMpScoreScreen>>,
) {
    for entity in &score_entities {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.try_despawn();
        }
    }
    commands.remove_resource::<MpRematchState>();
    commands.remove_resource::<super::score_stats::MatchStats>();
}

/// Tears down the active multiplayer session and returns to the main menu.
///
/// Shared by the score-screen, pause-menu, and disconnected-overlay disconnect
/// paths plus the score-screen Escape handler. `transport` is `Option`: the
/// disconnected-overlay path passes `None` (the peer is already gone, so there's
/// nothing to signal); every other path passes the live handle so the peer is
/// told to disconnect. Steam teardown happens BEFORE `connection.reset()` — once
/// `reset` zeroes `mode` to `Online`, downstream cleanup hooks can no longer tell
/// this was a Steam session.
#[allow(clippy::too_many_arguments)]
pub(super) fn do_mp_disconnect(
    connection: &mut NetworkConnection,
    transport: Option<&TransportHandle>,
    steam_client: Option<&bevy_steamworks::Client>,
    steam_lobby: Option<&mut crate::steam::multiplayer::SteamLobbyState>,
    steam_socket: Option<&mut crate::steam::multiplayer::SteamP2pSocket>,
    lobby: &mut MultiplayerLobby,
    commands: &mut Commands,
    next_app_state: &mut NextState<AppState>,
) {
    if let Some(transport) = transport {
        transport.send_command(TransportCommand::Disconnect);
    }
    crate::steam::multiplayer::shutdown_steam_session(steam_client, steam_lobby, steam_socket);
    connection.reset();
    commands.remove_resource::<MultiplayerSession>();
    // Disconnecting cancels any rematch. Clear PendingRematch defensively so a
    // confirm-rematch-on-the-same-frame-as-leave race can't carry it into the
    // main menu and then try to rematch on the session we just tore down.
    // (PendingRematch is only set on the normal both-ready path, which never
    // calls this helper, so this never affects a legitimate rematch.)
    commands.remove_resource::<PendingRematch>();
    // Going MultiplayerGame → MainMenu skips the WizardTower-exit hook that
    // normally resets the lobby, so any stale phase / peer_protocol_version from
    // this session would block reconnecting later. Reset here.
    *lobby = MultiplayerLobby::new();
    next_app_state.set(AppState::MainMenu);
}

/// Handles score screen button clicks.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_mp_score_buttons(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&MpScoreButtonAction>,
    mut rematch_state: ResMut<MpRematchState>,
    mut connection: ResMut<NetworkConnection>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut status_text: Query<&mut Text, With<RematchStatusText>>,
    transport: Option<Res<TransportHandle>>,
    steam_client: Option<Res<bevy_steamworks::Client>>,
    mut steam_lobby: Option<ResMut<crate::steam::multiplayer::SteamLobbyState>>,
    mut steam_socket: Option<ResMut<crate::steam::multiplayer::SteamP2pSocket>>,
    mut commands: Commands,
    mut lobby: ResMut<MultiplayerLobby>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                MpScoreButtonAction::Rematch => {
                    rematch_state.local_ready = true;
                    connection
                        .outgoing_messages
                        .push(NetworkMessage::RematchReady);

                    if let Ok(mut text) = status_text.single_mut() {
                        **text = "Waiting for opponent...".to_string();
                    }

                    if rematch_state.remote_ready {
                        commands.insert_resource(PendingRematch);
                        next_app_state.set(AppState::MainMenu);
                    }
                }
                MpScoreButtonAction::Disconnect => {
                    do_mp_disconnect(
                        &mut connection,
                        transport.as_deref(),
                        steam_client.as_deref(),
                        steam_lobby.as_deref_mut(),
                        steam_socket.as_deref_mut(),
                        &mut lobby,
                        &mut commands,
                        &mut next_app_state,
                    );
                }
            }
        }
    }
}

/// Escape on the multiplayer score screen disconnects and returns to the main
/// menu — the same teardown as the score screen's Disconnect button. Registered
/// under `in_mp_score_screen`, so it only fires on the score screen (and NOT in
/// `ButtonActionSet`: this reads the keyboard, not button clicks).
#[allow(clippy::too_many_arguments)]
pub(super) fn mp_score_escape_handler(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut connection: ResMut<NetworkConnection>,
    mut next_app_state: ResMut<NextState<AppState>>,
    transport: Option<Res<TransportHandle>>,
    steam_client: Option<Res<bevy_steamworks::Client>>,
    mut steam_lobby: Option<ResMut<crate::steam::multiplayer::SteamLobbyState>>,
    mut steam_socket: Option<ResMut<crate::steam::multiplayer::SteamP2pSocket>>,
    mut commands: Commands,
    mut lobby: ResMut<MultiplayerLobby>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }
    do_mp_disconnect(
        &mut connection,
        transport.as_deref(),
        steam_client.as_deref(),
        steam_lobby.as_deref_mut(),
        steam_socket.as_deref_mut(),
        &mut lobby,
        &mut commands,
        &mut next_app_state,
    );
}

/// Processes incoming network messages during the score screen.
pub(super) fn handle_mp_score_messages(
    mut connection: ResMut<NetworkConnection>,
    mut rematch_state: ResMut<MpRematchState>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut status_text: Query<&mut Text, With<RematchStatusText>>,
    mut match_stats: Option<ResMut<super::score_stats::MatchStats>>,
    mut commands: Commands,
) {
    if connection.incoming_messages.is_empty() {
        return;
    }

    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::RematchReady => {
                rematch_state.remote_ready = true;

                if let Ok(mut text) = status_text.single_mut() {
                    if rematch_state.local_ready {
                        **text = "Starting rematch...".to_string();
                    } else {
                        **text = "Opponent wants a rematch!".to_string();
                    }
                }

                if rematch_state.local_ready {
                    commands.insert_resource(PendingRematch);
                    next_app_state.set(AppState::MainMenu);
                }
            }
            NetworkMessage::GameOver { .. } => {
                // Already handled at the game-over transition; ignore here.
            }
            NetworkMessage::WizardStatsReport {
                spell_damage,
                spell_healing,
            } => {
                // Host receives the guest wizard's spell stats and fills in the
                // enemy column of its scoreboard (reactively updated by
                // `update_mp_stat_values`).
                if let Some(stats) = match_stats.as_mut() {
                    stats.enemy_damage = spell_damage;
                    stats.enemy_healing = spell_healing;
                }
            }
            // Keep connection/lobby-level messages for other handlers; DROP
            // stale in-flight gameplay messages (weather, spell hits, avatar
            // control, etc.) — the match is over, so re-queuing them forever
            // just wastes a drain every frame.
            other @ (NetworkMessage::Ping { .. }
            | NetworkMessage::Pong { .. }
            | NetworkMessage::HandshakeVersion { .. }) => unhandled.push(other),
            _ => {}
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}

/// Detects unexpected connection loss and transitions to the Disconnected overlay.
///
/// Only triggers on `Failed` state — intentional disconnects are handled by button actions.
pub(super) fn detect_mp_disconnect(
    connection: Res<NetworkConnection>,
    session: Option<Res<MultiplayerSession>>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    mut next_mp_state: ResMut<NextState<MultiplayerGameState>>,
) {
    // Only check if we still have an active session — avoids double-triggering
    // after an intentional disconnect already queued a state transition.
    if session.is_none() {
        return;
    }

    // Don't re-trigger if already on the Disconnected screen
    if mp_state.is_some_and(|s| *s.get() == MultiplayerGameState::Disconnected) {
        return;
    }

    // Both Failed (transport error) and Disconnected (peer-initiated graceful
    // close) are treated as unexpected loss here. The transport emits
    // `StateChanged(Disconnected)` after the peer closes cleanly, and without
    // this we'd silently leave the player stuck in a live match with a dead
    // socket.
    if matches!(
        connection.state,
        ConnectionState::Failed | ConnectionState::Disconnected
    ) {
        next_mp_state.set(MultiplayerGameState::Disconnected);
    }
}

/// Detects connection loss while we're still in `MultiplayerLoading` (before
/// the `MultiplayerGame` sub-state machine exists) and bails out cleanly.
///
/// Without this, a disconnect during loading would let `process_mp_spawn_queue`
/// keep ticking against a dead connection until the queue completed and the
/// game transitioned to `MultiplayerGame` — at which point `detect_mp_disconnect`
/// would finally fire. This system shortcuts that by sending the player
/// straight back to the wizard tower with the connection torn down.
#[allow(clippy::too_many_arguments)]
pub(super) fn detect_mp_loading_disconnect(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    session: Option<Res<MultiplayerSession>>,
    transport: Option<Res<TransportHandle>>,
    mut next_app_state: ResMut<NextState<crate::state::AppState>>,
    steam_client: Option<Res<bevy_steamworks::Client>>,
    mut steam_lobby: Option<ResMut<crate::steam::multiplayer::SteamLobbyState>>,
    mut steam_socket: Option<ResMut<crate::steam::multiplayer::SteamP2pSocket>>,
    // Despawn any battlefield, units, terrain, or screen-tagged UI that the
    // spawn queue produced before the disconnect hit. Without these the 3D
    // meshes leak into the wizard tower scene after we transition.
    gameplay_entities: Query<
        Entity,
        Or<(
            With<crate::game::components::OnGameplayScreen>,
            With<super::components::OnMultiplayerGameScreen>,
        )>,
    >,
) {
    // Mirror `detect_mp_disconnect`'s guard: a missing session means we're
    // not actually in a live match — the `Disconnected` default-state could
    // otherwise spuriously fire this handler on the first frame of loading.
    if session.is_none() {
        return;
    }

    if matches!(
        connection.state,
        ConnectionState::Failed | ConnectionState::Disconnected
    ) {
        warn!("[MP] Connection lost during loading; returning to wizard tower.");
        if let Some(t) = transport.as_ref() {
            t.send_command(TransportCommand::Disconnect);
        }
        // Steam teardown must run BEFORE connection.reset() so we can still
        // see `mode == Steam` and find the SteamLobbyState we need to leave.
        crate::steam::multiplayer::shutdown_steam_session(
            steam_client.as_deref(),
            steam_lobby.as_deref_mut(),
            steam_socket.as_deref_mut(),
        );
        // Clean up everything the loading queue had already spawned —
        // `OnExit(MultiplayerLoading)` only tears down the loading screen
        // and queue resources, so without these despawns the partially-built
        // arena would leak into MetaGame.
        for entity in &gameplay_entities {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.try_despawn();
            }
        }
        commands.remove_resource::<crate::game::pathfinding::resources::PathfindingGrid>();
        connection.reset();
        commands.remove_resource::<MultiplayerSession>();
        // `MetaGameState` is a SubState of `AppState::MetaGame` and is
        // re-initialised to its `#[default] = WizardTower` whenever the
        // parent state enters — no explicit `next_meta_state.set` needed.
        next_app_state.set(crate::state::AppState::MetaGame);
    }
}

// ── Escape Key Handler ──────────────────────────────────────────────

/// Toggles the escape menu overlay during multiplayer gameplay.
pub(super) fn mp_escape_key_handler(
    keyboard: Res<ButtonInput<KeyCode>>,
    mp_state: Option<Res<State<MultiplayerGameState>>>,
    mut next_mp_state: ResMut<NextState<MultiplayerGameState>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    let Some(state) = mp_state else { return };

    match *state.get() {
        MultiplayerGameState::Running => {
            next_mp_state.set(MultiplayerGameState::Paused);
        }
        MultiplayerGameState::Paused
        | MultiplayerGameState::SpellBook
        | MultiplayerGameState::CauldronMenu => {
            next_mp_state.set(MultiplayerGameState::Running);
        }
        MultiplayerGameState::Settings => {
            next_mp_state.set(MultiplayerGameState::Paused);
        }
        _ => {}
    }
}

// ── Escape Menu (Paused Overlay) ────────────────────────────────────

const PAUSE_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 250.0,
    height: 65.0,
    border_width: 3.0,
    font_size: 20.0,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};

/// Spawns the MP escape menu overlay.
pub(super) fn setup_mp_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.7)),
            GlobalZIndex(500),
            OnMpPauseScreen,
            OnMultiplayerGameScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Menu"),
                TextFont::from_font_size(40.0),
                TextColor(TEXT_PRIMARY),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            spawn_button(
                parent,
                "Resume",
                MpPauseButtonAction::Resume,
                &PAUSE_BUTTON_STYLE,
            );
            spawn_button(
                parent,
                "Settings",
                MpPauseButtonAction::Settings,
                &PAUSE_BUTTON_STYLE,
            );
            spawn_button(
                parent,
                "Disconnect",
                MpPauseButtonAction::Disconnect,
                &PAUSE_BUTTON_STYLE,
            );
        });
}

/// Cleans up the MP escape menu overlay.
pub(super) fn cleanup_mp_pause_menu(
    mut commands: Commands,
    entities: Query<Entity, With<OnMpPauseScreen>>,
) {
    for entity in &entities {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.try_despawn();
        }
    }
}

/// Handles MP escape menu button clicks.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_mp_pause_buttons(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&MpPauseButtonAction>,
    mut connection: ResMut<NetworkConnection>,
    mut next_mp_state: ResMut<NextState<MultiplayerGameState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    transport: Option<Res<TransportHandle>>,
    mut commands: Commands,
    steam_client: Option<Res<bevy_steamworks::Client>>,
    mut steam_lobby: Option<ResMut<crate::steam::multiplayer::SteamLobbyState>>,
    mut steam_socket: Option<ResMut<crate::steam::multiplayer::SteamP2pSocket>>,
    mut lobby: ResMut<MultiplayerLobby>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                MpPauseButtonAction::Resume => {
                    next_mp_state.set(MultiplayerGameState::Running);
                }
                MpPauseButtonAction::Settings => {
                    next_mp_state.set(MultiplayerGameState::Settings);
                }
                MpPauseButtonAction::Disconnect => {
                    do_mp_disconnect(
                        &mut connection,
                        transport.as_deref(),
                        steam_client.as_deref(),
                        steam_lobby.as_deref_mut(),
                        steam_socket.as_deref_mut(),
                        &mut lobby,
                        &mut commands,
                        &mut next_app_state,
                    );
                }
            }
        }
    }
}

// ── Disconnected Overlay ────────────────────────────────────────────

/// Spawns the disconnected overlay informing the player of connection loss.
pub(super) fn setup_mp_disconnected(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.85)),
            GlobalZIndex(600),
            OnMpDisconnectedScreen,
            OnMultiplayerGameScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Disconnected"),
                TextFont::from_font_size(48.0),
                TextColor(Color::srgb(0.95, 0.3, 0.3)),
            ));

            parent.spawn((
                Text::new("The connection to your opponent was lost."),
                TextFont::from_font_size(20.0),
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            spawn_button(
                parent,
                "Return to Menu",
                MpDisconnectedButtonAction,
                &PAUSE_BUTTON_STYLE,
            );
        });
}

/// Cleans up the disconnected overlay.
pub(super) fn cleanup_mp_disconnected(
    mut commands: Commands,
    entities: Query<Entity, With<OnMpDisconnectedScreen>>,
) {
    for entity in &entities {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.try_despawn();
        }
    }
}

/// Handles the Return to Menu button on the disconnected overlay.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_mp_disconnected_buttons(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&MpDisconnectedButtonAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    steam_client: Option<Res<bevy_steamworks::Client>>,
    mut steam_lobby: Option<ResMut<crate::steam::multiplayer::SteamLobbyState>>,
    mut steam_socket: Option<ResMut<crate::steam::multiplayer::SteamP2pSocket>>,
    mut lobby: ResMut<MultiplayerLobby>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            // `None` transport: the overlay only appears when the peer is already
            // gone, so there's nothing to signal — just tear down locally.
            do_mp_disconnect(
                &mut connection,
                None,
                steam_client.as_deref(),
                steam_lobby.as_deref_mut(),
                steam_socket.as_deref_mut(),
                &mut lobby,
                &mut commands,
                &mut next_app_state,
            );
            return;
        }
    }
}
