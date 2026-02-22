//! Multiplayer game plugin.
//!
//! Registers only multiplayer-specific systems. The host reuses all single-player
//! gameplay systems (movement, combat, spells, etc.) via `is_gameplay_running`,
//! so this plugin only adds networking, guest rendering, score screen, and
//! disconnect detection.

use bevy::prelude::*;

use crate::game::cauldron::resources::CauldronBuffs;
use crate::game::input::messages::MouseClicked;
use crate::game::plugin::{GlobalAttackCycle, PostCombatSet};
use crate::game::resources::{GameOutcome, KillStats};
use crate::networking::entity_map::EntityIdCounter;
use crate::networking::entity_map::NetworkEntityMap;
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{ConnectionState, NetworkConnection};
use crate::networking::session::{is_multiplayer_guest, is_multiplayer_host, MultiplayerSession};
use crate::networking::snapshot::SnapshotTick;
use crate::state::{AppState, MultiplayerGameState};
use crate::ui::components::ButtonStyle;
use crate::ui::plugin::ButtonActionSet;
use crate::ui::systems::spawn_button;

use bevy::input::keyboard::KeyCode;

use super::components::{
    MpDisconnectedButtonAction, MpPauseButtonAction, MpRematchState, MpScoreButtonAction,
    OnMpDisconnectedScreen, OnMpPauseScreen, OnMpScoreScreen, OnMultiplayerGameScreen,
    PendingRematch, RematchStatusText,
};
use super::guest_systems;
use super::host_systems;
use super::loading;
use super::spell_commands;

use crate::game::units::infantry::components::DefendersActivated;

/// Safe run condition for `MultiplayerGameState` sub-states.
///
/// Returns true for both Running and Paused states, since the escape menu
/// overlay does not pause gameplay. Unlike `in_state(MultiplayerGameState::*)`,
/// these won't panic if the sub-state resource has already been removed.
fn in_mp_running(state: Option<Res<State<MultiplayerGameState>>>) -> bool {
    state.is_some_and(|s| {
        matches!(
            *s.get(),
            MultiplayerGameState::Running
                | MultiplayerGameState::Paused
                | MultiplayerGameState::SpellBook
        )
    })
}

fn in_mp_score_screen(state: Option<Res<State<MultiplayerGameState>>>) -> bool {
    state.is_some_and(|s| *s.get() == MultiplayerGameState::ScoreScreen)
}

fn in_mp_paused(state: Option<Res<State<MultiplayerGameState>>>) -> bool {
    state.is_some_and(|s| *s.get() == MultiplayerGameState::Paused)
}

fn in_mp_disconnected(state: Option<Res<State<MultiplayerGameState>>>) -> bool {
    state.is_some_and(|s| *s.get() == MultiplayerGameState::Disconnected)
}

/// Plugin that manages multiplayer gameplay.
///
/// The host reuses all single-player gameplay systems (registered by `GamePlugin`,
/// `UnitsPlugin`, `InfantryPlugin`, `ArcherPlugin`, `KingPlugin`, etc.) via the
/// `is_gameplay_running` run condition. This plugin only adds:
/// - Loading and camera setup
/// - Resource init/cleanup
/// - Host networking (ID assignment, snapshots, king death check)
/// - Guest snapshot rendering and game-over handling
/// - Score screen UI and rematch flow
/// - Disconnect detection
pub struct MultiplayerGamePlugin;

impl Plugin for MultiplayerGamePlugin {
    fn build(&self, app: &mut App) {
        // ── Multiplayer Loading ──────────────────────────────────────
        app.add_systems(
            OnEnter(AppState::MultiplayerLoading),
            loading::init_mp_loading,
        )
        .add_systems(
            Update,
            loading::process_mp_spawn_queue.run_if(in_state(AppState::MultiplayerLoading)),
        )
        .add_systems(
            OnExit(AppState::MultiplayerLoading),
            loading::cleanup_mp_loading,
        );

        // ── Camera ───────────────────────────────────────────────────
        app.add_systems(
            OnEnter(AppState::MultiplayerGame),
            loading::setup_mp_camera,
        );
        app.add_systems(
            OnExit(AppState::MultiplayerGame),
            loading::restore_camera,
        );

        // ── Resource Init / Cleanup ──────────────────────────────────
        app.add_systems(OnEnter(AppState::MultiplayerGame), init_mp_game);
        app.add_systems(OnExit(AppState::MultiplayerGame), cleanup_mp_game);

        // ── Host: MP King Death Check ────────────────────────────────
        // Replaces SP's check_win_lose_conditions during multiplayer.
        // Runs after PostCombatSet so corpses have been created.
        let mp_host = in_mp_running.and(is_multiplayer_host);

        app.add_systems(
            Update,
            host_systems::check_mp_king_death
                .after(PostCombatSet)
                .run_if(mp_host.clone()),
        );

        // ── Host Networking: ID Assignment + Snapshots ───────────────
        app.add_systems(
            Update,
            (
                host_systems::assign_network_ids,
                host_systems::collect_spell_effect_snapshots,
                host_systems::collect_spell_projectile_snapshots,
                host_systems::send_state_snapshots,
            )
                .chain()
                .after(PostCombatSet)
                .run_if(mp_host),
        );

        // ── Guest: Snapshot Rendering ────────────────────────────────
        app.add_systems(
            Update,
            (
                guest_systems::apply_state_snapshot,
                guest_systems::apply_spell_snapshot,
            )
                .chain()
                .run_if(in_mp_running.and(is_multiplayer_guest)),
        );

        // ── Guest: Game Over Message ──────────────────────────────────
        app.add_systems(
            Update,
            guest_systems::handle_game_over_message
                .run_if(in_mp_running.and(is_multiplayer_guest)),
        );

        // ── Guest: Spell Input Sending ──────────────────────────────────
        let mp_guest = in_mp_running.and(is_multiplayer_guest);
        app.add_systems(
            Update,
            (
                spell_commands::send_guest_prime_spell,
                spell_commands::send_guest_spell_commands,
            )
                .run_if(mp_guest),
        );

        // ── Host: Guest Spell Command Processing ──────────────────────
        // Only processes network messages into GuestInputState/GuestCursorPosition.
        // The actual casting is handled by the widened SP spell systems.
        let mp_host_spells = in_mp_running.and(is_multiplayer_host);
        app.add_systems(
            Update,
            spell_commands::process_guest_spell_commands
                .run_if(mp_host_spells),
        );

        // ── Escape Key (Running → Paused toggle) ──────────────────────
        app.add_systems(
            Update,
            mp_escape_key_handler.run_if(
                in_state(AppState::MultiplayerGame)
                    .and(in_mp_running.or(in_mp_paused)),
            ),
        );

        // ── Escape Menu (Paused overlay) ──────────────────────────────
        app.add_systems(
            OnEnter(MultiplayerGameState::Paused),
            setup_mp_pause_menu,
        );
        app.add_systems(
            OnExit(MultiplayerGameState::Paused),
            cleanup_mp_pause_menu,
        );
        app.add_systems(
            Update,
            handle_mp_pause_buttons
                .in_set(ButtonActionSet)
                .run_if(in_mp_paused),
        );

        // ── Disconnected Overlay ──────────────────────────────────────
        app.add_systems(
            OnEnter(MultiplayerGameState::Disconnected),
            setup_mp_disconnected,
        );
        app.add_systems(
            OnExit(MultiplayerGameState::Disconnected),
            cleanup_mp_disconnected,
        );
        app.add_systems(
            Update,
            handle_mp_disconnected_buttons
                .in_set(ButtonActionSet)
                .run_if(in_mp_disconnected),
        );

        // ── Score Screen ──────────────────────────────────────────────
        app.add_systems(
            OnEnter(MultiplayerGameState::ScoreScreen),
            setup_mp_score_screen,
        );
        app.add_systems(
            OnExit(MultiplayerGameState::ScoreScreen),
            cleanup_mp_score_screen,
        );
        app.add_systems(
            Update,
            (
                handle_mp_score_buttons.in_set(ButtonActionSet),
                handle_mp_score_messages,
            )
                .run_if(in_mp_score_screen),
        );

        // ── Disconnect Detection ──────────────────────────────────────
        app.add_systems(
            Update,
            detect_mp_disconnect.run_if(in_state(AppState::MultiplayerGame)),
        );
    }
}

/// Initializes resources needed for multiplayer gameplay.
///
/// GlobalAttackCycle, KillStats, DefendersActivated, and GameOutcome are already
/// initialized by GamePlugin at startup. We reset them here to ensure clean state
/// and additionally insert MP-only resources.
fn init_mp_game(
    mut commands: Commands,
    mut attack_cycle: ResMut<GlobalAttackCycle>,
    mut kill_stats: ResMut<KillStats>,
    mut defenders_activated: ResMut<DefendersActivated>,
    mut game_outcome: ResMut<GameOutcome>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Reset globally-owned resources to clean state for this match
    *attack_cycle = GlobalAttackCycle::default();
    kill_stats.reset();
    defenders_activated.active = true;
    *game_outcome = GameOutcome::Victory;

    // Insert MP-only resources
    commands.init_resource::<EntityIdCounter>();
    commands.init_resource::<NetworkEntityMap>();
    commands.init_resource::<SnapshotTick>();
    commands.init_resource::<spell_commands::GuestCursorPosition>();
    commands.init_resource::<spell_commands::GuestInputState>();
    commands.init_resource::<crate::networking::snapshot::SpellSnapshotData>();
    commands.init_resource::<super::components::SpellEffectEntityMap>();
    commands.init_resource::<guest_systems::LatestSnapshot>();

    // Preload ghost spell assets for guest rendering
    let ghost_assets = super::components::GhostSpellAssets::new(&mut meshes, &mut materials);
    commands.insert_resource(ghost_assets);

    // Preload spell effect assets for guest-side spell rendering
    let spell_effect_assets =
        super::components::SpellEffectAssets::new(&mut meshes, &mut materials);
    commands.insert_resource(spell_effect_assets);
}

/// Cleans up multiplayer game entities and resources.
///
/// If `PendingRematch` is present, the `MultiplayerSession` is kept alive
/// so the WebRTC connection persists through the rematch flow.
#[allow(clippy::too_many_arguments)]
fn cleanup_mp_game(
    mut commands: Commands,
    mp_entities: Query<Entity, With<OnMultiplayerGameScreen>>,
    pending_rematch: Option<Res<PendingRematch>>,
    mut attack_cycle: ResMut<GlobalAttackCycle>,
    mut kill_stats: ResMut<KillStats>,
    mut defenders_activated: ResMut<DefendersActivated>,
    mut cauldron_buffs: ResMut<CauldronBuffs>,
    mut game_outcome: ResMut<GameOutcome>,
) {
    for entity in &mp_entities {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
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

    // Remove MP-only resources that are created in init_mp_game.
    commands.remove_resource::<EntityIdCounter>();
    commands.remove_resource::<NetworkEntityMap>();
    commands.remove_resource::<SnapshotTick>();
    commands.remove_resource::<spell_commands::GuestCursorPosition>();
    commands.remove_resource::<spell_commands::GuestInputState>();
    commands.remove_resource::<super::components::GhostSpellAssets>();
    commands.remove_resource::<super::components::SpellEffectAssets>();
    commands.remove_resource::<super::components::SpellEffectEntityMap>();
    commands.remove_resource::<crate::networking::snapshot::SpellSnapshotData>();
    commands.remove_resource::<guest_systems::LatestSnapshot>();

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
    background: Color::hsla(0.0, 0.0, 0.15, 1.0),
    border: Color::hsla(0.0, 0.0, 0.3, 1.0),
    text_color: Color::hsla(0.0, 0.0, 0.9, 1.0),
};

// ── Score Screen Systems ──────────────────────────────────────────────

/// Spawns the multiplayer score screen UI.
fn setup_mp_score_screen(mut commands: Commands, game_outcome: Res<GameOutcome>) {
    commands.init_resource::<MpRematchState>();

    let title_text = match *game_outcome {
        GameOutcome::Victory => "VICTORY",
        GameOutcome::Defeat | GameOutcome::DefeatKingDied => "DEFEAT",
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
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

            // Buttons
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(15.0),
                    margin: UiRect::top(Val::Px(20.0)),
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
                });

            // Status text
            parent.spawn((
                RematchStatusText,
                Text::new(""),
                TextFont::from_font_size(18.0),
                TextColor(SCORE_TEXT_COLOR),
            ));
        });
}

/// Cleans up score screen entities and resources.
fn cleanup_mp_score_screen(
    mut commands: Commands,
    score_entities: Query<Entity, With<OnMpScoreScreen>>,
) {
    for entity in &score_entities {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
    commands.remove_resource::<MpRematchState>();
}

/// Handles score screen button clicks.
fn handle_mp_score_buttons(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&MpScoreButtonAction>,
    mut rematch_state: ResMut<MpRematchState>,
    mut connection: ResMut<NetworkConnection>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut status_text: Query<&mut Text, With<RematchStatusText>>,
    mut commands: Commands,
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
                    #[cfg(target_arch = "wasm32")]
                    crate::networking::webrtc::disconnect();
                    connection.state = ConnectionState::Disconnected;
                    commands.remove_resource::<MultiplayerSession>();
                    next_app_state.set(AppState::MainMenu);
                }
            }
        }
    }
}

/// Processes incoming network messages during the score screen.
fn handle_mp_score_messages(
    mut connection: ResMut<NetworkConnection>,
    mut rematch_state: ResMut<MpRematchState>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut status_text: Query<&mut Text, With<RematchStatusText>>,
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
            NetworkMessage::GameOver(_) => {
                // Already handled, ignore
            }
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}

/// Detects unexpected connection loss and transitions to the Disconnected overlay.
///
/// Only triggers on `Failed` state — intentional disconnects are handled by button actions.
fn detect_mp_disconnect(
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

    if connection.state == ConnectionState::Failed {
        next_mp_state.set(MultiplayerGameState::Disconnected);
    }
}

// ── Escape Key Handler ──────────────────────────────────────────────

/// Toggles the escape menu overlay during multiplayer gameplay.
fn mp_escape_key_handler(
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
        MultiplayerGameState::Paused | MultiplayerGameState::SpellBook => {
            next_mp_state.set(MultiplayerGameState::Running);
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
    background: Color::hsla(0.0, 0.0, 0.15, 1.0),
    border: Color::hsla(0.0, 0.0, 0.3, 1.0),
    text_color: Color::hsla(0.0, 0.0, 0.9, 1.0),
};

/// Spawns the MP escape menu overlay.
fn setup_mp_pause_menu(mut commands: Commands) {
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
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
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
                "Disconnect",
                MpPauseButtonAction::Disconnect,
                &PAUSE_BUTTON_STYLE,
            );
        });
}

/// Cleans up the MP escape menu overlay.
fn cleanup_mp_pause_menu(
    mut commands: Commands,
    entities: Query<Entity, With<OnMpPauseScreen>>,
) {
    for entity in &entities {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
}

/// Handles MP escape menu button clicks.
fn handle_mp_pause_buttons(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&MpPauseButtonAction>,
    mut connection: ResMut<NetworkConnection>,
    mut next_mp_state: ResMut<NextState<MultiplayerGameState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                MpPauseButtonAction::Resume => {
                    next_mp_state.set(MultiplayerGameState::Running);
                }
                MpPauseButtonAction::Disconnect => {
                    #[cfg(target_arch = "wasm32")]
                    crate::networking::webrtc::disconnect();
                    connection.state = ConnectionState::Disconnected;
                    commands.remove_resource::<MultiplayerSession>();
                    next_app_state.set(AppState::MainMenu);
                }
            }
        }
    }
}

// ── Disconnected Overlay ────────────────────────────────────────────

/// Spawns the disconnected overlay informing the player of connection loss.
fn setup_mp_disconnected(mut commands: Commands) {
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
fn cleanup_mp_disconnected(
    mut commands: Commands,
    entities: Query<Entity, With<OnMpDisconnectedScreen>>,
) {
    for entity in &entities {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
}

/// Handles the Return to Menu button on the disconnected overlay.
fn handle_mp_disconnected_buttons(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&MpDisconnectedButtonAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            commands.remove_resource::<MultiplayerSession>();
            next_app_state.set(AppState::MainMenu);
        }
    }
}
