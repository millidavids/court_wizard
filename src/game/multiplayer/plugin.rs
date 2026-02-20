//! Multiplayer game plugin.
//!
//! Registers all multiplayer gameplay systems. This is completely independent
//! from `GamePlugin` — it reuses shared helper functions but has its own
//! system registrations under `AppState::MultiplayerGame`.

use bevy::prelude::*;

use crate::game::cauldron::resources::CauldronBuffs;
use crate::game::input::messages::MouseClicked;
use crate::game::plugin::GlobalAttackCycle;
use crate::game::resources::{GameOutcome, KillStats};
use crate::game::run_conditions::any_exist;
use crate::game::shared_systems;
use crate::game::units::archer::components::{Archer, Arrow};
use crate::game::units::archer::systems as archer_systems;
use crate::game::units::components::{
    BattleHymnModifier, BerserkerRageModifier, FogEvasionModifier, GreaseSlipModifier,
    MarkedForDeathModifier, MesmerizedModifier, SleepModifier,
};
use crate::game::units::infantry::components::DefendersActivated;
use crate::game::units::infantry::systems as infantry_systems;
use crate::game::units::infantry::Infantry;
use crate::game::units::king::components::King;
use crate::game::units::king::systems as king_systems;
use crate::game::units::movement;
use crate::game::units::systems as unit_systems;
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

use super::components::{
    MpRematchState, MpScoreButtonAction, OnMpScoreScreen, OnMultiplayerGameScreen,
    PendingRematch, RematchStatusText,
};
use super::guest_systems;
use super::host_systems;
use super::loading;

/// Multiplayer-specific system sets for ordering host simulation.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MpSystemSet {
    /// Velocity/targeting calculations (parallel, immutable queries).
    Velocity,
    /// Movement application (after velocity).
    Movement,
}

/// Plugin that manages multiplayer gameplay.
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

        // ── System Set Configuration ─────────────────────────────────
        let mp_host = in_state(MultiplayerGameState::Running).and(is_multiplayer_host);

        app.configure_sets(
            Update,
            (
                MpSystemSet::Velocity.run_if(mp_host.clone()),
                MpSystemSet::Movement
                    .run_if(mp_host.clone())
                    .after(MpSystemSet::Velocity),
            ),
        );

        // ── Host Simulation: Core Game Loop ──────────────────────────
        // Tick timers
        app.add_systems(
            Update,
            (
                shared_systems::tick_attack_cycle,
                shared_systems::tick_elapsed_time,
            )
                .run_if(mp_host.clone()),
        );

        // Velocity set: activation, separation, wall avoidance
        app.add_systems(
            Update,
            (
                shared_systems::activate_defenders_on_proximity,
                shared_systems::apply_separation,
                shared_systems::apply_wall_avoidance,
            )
                .chain()
                .in_set(MpSystemSet::Velocity),
        );

        // Between velocity and movement: effectiveness, terrain slowdown
        app.add_systems(
            Update,
            (
                shared_systems::calculate_effectiveness,
                shared_systems::apply_rough_terrain_slowdown,
            )
                .chain()
                .run_if(mp_host.clone())
                .after(MpSystemSet::Velocity)
                .before(MpSystemSet::Movement),
        );

        // After movement: wall collision, combat, corpse conversion, billboards
        app.add_systems(
            Update,
            (
                shared_systems::enforce_wall_collision,
                shared_systems::combat,
                shared_systems::convert_dead_to_corpses,
            )
                .chain()
                .run_if(mp_host.clone())
                .after(MpSystemSet::Movement),
        );

        // ── Host Simulation: Unit Systems ────────────────────────────
        // Modifier tickers + damage effects
        app.add_systems(
            Update,
            (
                unit_systems::process_pending_damage_effects,
                unit_systems::update_temporary_hit_points,
                unit_systems::update_frost_slow_modifiers,
                unit_systems::update_rooted_modifiers,
                unit_systems::update_haste_modifiers,
                unit_systems::update_spike_growth_slow_modifiers,
                unit_systems::update_fire_dot,
                unit_systems::update_electric_charge,
                unit_systems::update_electric_arc_visuals,
                unit_systems::update_persistent_effect_visuals,
            )
                .run_if(mp_host.clone()),
        );

        // Conditional modifier tickers (only when components exist)
        app.add_systems(
            Update,
            (
                unit_systems::update_mark_of_death_modifiers
                    .run_if(any_with_component::<MarkedForDeathModifier>),
                unit_systems::update_mesmerized_modifiers
                    .run_if(any_with_component::<MesmerizedModifier>),
                unit_systems::update_sleep_modifiers
                    .run_if(any_with_component::<SleepModifier>),
                unit_systems::update_battle_hymn_modifiers
                    .run_if(any_with_component::<BattleHymnModifier>),
                unit_systems::update_berserker_rage_modifiers
                    .run_if(any_with_component::<BerserkerRageModifier>),
                unit_systems::update_fog_evasion_modifiers
                    .run_if(any_with_component::<FogEvasionModifier>),
                unit_systems::update_grease_slip_modifiers
                    .run_if(any_with_component::<GreaseSlipModifier>),
            )
                .run_if(mp_host.clone()),
        );

        // Unit movement application (after movement calculations)
        app.add_systems(
            Update,
            movement::apply_unit_movement
                .run_if(mp_host.clone())
                .after(MpSystemSet::Movement),
        );

        // ── Host Simulation: Infantry ────────────────────────────────
        app.add_systems(
            Update,
            (
                infantry_systems::check_defender_activation
                    .before(MpSystemSet::Velocity),
                infantry_systems::update_infantry_targeting
                    .in_set(MpSystemSet::Velocity),
                infantry_systems::infantry_movement
                    .in_set(MpSystemSet::Movement),
            )
                .run_if(any_exist::<Infantry>())
                .run_if(mp_host.clone()),
        );

        // ── Host Simulation: Archer ──────────────────────────────────
        app.add_systems(
            Update,
            (
                archer_systems::update_archer_targeting
                    .in_set(MpSystemSet::Velocity),
                archer_systems::archer_movement
                    .in_set(MpSystemSet::Movement),
                (
                    archer_systems::update_archer_movement_timers,
                    archer_systems::archer_melee_combat,
                    archer_systems::archer_ranged_combat,
                )
                    .chain(),
            )
                .run_if(any_exist::<Archer>())
                .run_if(mp_host.clone()),
        );

        app.add_systems(
            Update,
            (
                archer_systems::move_arrows,
                archer_systems::check_arrow_collisions,
            )
                .chain()
                .run_if(any_exist::<Arrow>())
                .run_if(mp_host.clone()),
        );

        // ── Host Simulation: King ────────────────────────────────────
        app.add_systems(
            Update,
            (
                king_systems::update_king_targeting
                    .in_set(MpSystemSet::Velocity),
                king_systems::king_movement
                    .in_set(MpSystemSet::Movement),
                king_systems::king_cohesion_force
                    .after(MpSystemSet::Velocity)
                    .before(MpSystemSet::Movement),
                king_systems::snap_kings_guard_to_king
                    .in_set(MpSystemSet::Movement),
            )
                .run_if(any_exist::<King>())
                .run_if(mp_host.clone()),
        );

        // ── Host Networking: ID Assignment + Snapshots ───────────────
        app.add_systems(
            Update,
            (
                host_systems::assign_network_ids,
                host_systems::send_state_snapshots,
            )
                .chain()
                .run_if(mp_host.clone()),
        );

        // ── Billboards (both host and guest) ─────────────────────────
        app.add_systems(
            Update,
            crate::game::systems::update_billboards
                .run_if(in_state(MultiplayerGameState::Running)),
        );

        // ── Guest: Snapshot Rendering ────────────────────────────────
        app.add_systems(
            Update,
            guest_systems::apply_state_snapshot
                .run_if(in_state(MultiplayerGameState::Running).and(is_multiplayer_guest)),
        );

        // ── Host: Win/Lose Detection ──────────────────────────────────
        app.add_systems(
            Update,
            host_systems::check_mp_king_death
                .run_if(mp_host)
                .after(shared_systems::convert_dead_to_corpses),
        );

        // ── Guest: Game Over Message ──────────────────────────────────
        app.add_systems(
            Update,
            guest_systems::handle_game_over_message
                .run_if(in_state(MultiplayerGameState::Running).and(is_multiplayer_guest)),
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
                .run_if(in_state(MultiplayerGameState::ScoreScreen)),
        );

        // ── Disconnect Detection ──────────────────────────────────────
        app.add_systems(
            Update,
            detect_mp_disconnect.run_if(in_state(AppState::MultiplayerGame)),
        );
    }
}

/// Initializes resources needed for multiplayer gameplay.
fn init_mp_game(mut commands: Commands) {
    commands.init_resource::<GlobalAttackCycle>();
    commands.init_resource::<KillStats>();
    commands.insert_resource(DefendersActivated { active: true });
    commands.init_resource::<EntityIdCounter>();
    commands.init_resource::<NetworkEntityMap>();
    commands.init_resource::<SnapshotTick>();
    commands.init_resource::<CauldronBuffs>();
    commands.insert_resource(GameOutcome::Victory);
}

/// Cleans up multiplayer game entities and resources.
///
/// If `PendingRematch` is present, the `MultiplayerSession` is kept alive
/// so the WebRTC connection persists through the rematch flow.
fn cleanup_mp_game(
    mut commands: Commands,
    mp_entities: Query<Entity, With<OnMultiplayerGameScreen>>,
    pending_rematch: Option<Res<PendingRematch>>,
) {
    for entity in &mp_entities {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
    commands.remove_resource::<GlobalAttackCycle>();
    commands.remove_resource::<KillStats>();
    commands.remove_resource::<DefendersActivated>();
    commands.remove_resource::<EntityIdCounter>();
    commands.remove_resource::<NetworkEntityMap>();
    commands.remove_resource::<SnapshotTick>();
    commands.remove_resource::<CauldronBuffs>();
    commands.remove_resource::<GameOutcome>();

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
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(SCORE_TITLE_COLOR),
            ));

            // Subtitle for King death
            if *game_outcome == GameOutcome::DefeatKingDied {
                parent.spawn((
                    Text::new("Your King was slain!"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
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
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
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

/// Detects connection loss during multiplayer gameplay and returns to main menu.
fn detect_mp_disconnect(
    connection: Res<NetworkConnection>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    if matches!(
        connection.state,
        ConnectionState::Failed | ConnectionState::Disconnected
    ) {
        commands.remove_resource::<MultiplayerSession>();
        next_app_state.set(AppState::MainMenu);
    }
}
