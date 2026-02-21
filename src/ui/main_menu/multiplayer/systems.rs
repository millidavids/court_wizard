//! Multiplayer screen systems.

use bevy::prelude::*;

use crate::config::WizardType;
use crate::config::save_data;
use crate::game::input::messages::MouseClicked;
use crate::game::units::wizard::components::Spell;
use crate::game::multiplayer::components::PendingRematch;
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{ConnectionState, NetworkConnection, PeerRole};
use crate::networking::session::MultiplayerSession;
use crate::state::{AppState, MenuState};
use crate::ui::components::ButtonColors;
use crate::ui::systems::spawn_button;

use super::super::wizard_select_shared::{self as shared};
use super::components::{
    ActiveConnectionButtons, CodeDisplayText, InitialButtons, LobbyPhase,
    MultiplayerButtonAction, OnMultiplayerScreen, PasteResponseButton, PingText,
    ReadyButtonArea, SignalingButtons, StatusText, WizardSelectScreen,
};
use super::constants::*;

/// Parses unlocked wizard types from save data string names.
fn parse_wizard_types(names: &[String]) -> Vec<WizardType> {
    names
        .iter()
        .filter_map(|name| match name.as_str() {
            "BoringOleMage" => Some(WizardType::BoringOleMage),
            "RuneCaster" => Some(WizardType::RuneCaster),
            "Randomancer" => Some(WizardType::Randomancer),
            "Arcanorouter" => Some(WizardType::Arcanorouter),
            _ => None,
        })
        .collect()
}

/// Parses unlocked spells from save data string names.
fn parse_spells(names: &[String]) -> Vec<Spell> {
    names
        .iter()
        .filter_map(|name| {
            Spell::all()
                .iter()
                .find(|s| format!("{:?}", s) == *name)
                .copied()
        })
        .collect()
}

/// Loads this player's unlocked wizard types and spells from save data.
fn load_my_unlocked_content() -> (Vec<WizardType>, Vec<Spell>) {
    let save = save_data::load_unified_save();
    if let Some(save) = &save {
        let wt = parse_wizard_types(&save.player.unlocked_content.wizard_types);
        let sp = parse_spells(&save.player.unlocked_content.spells);
        (wt, sp)
    } else {
        (
            vec![WizardType::BoringOleMage],
            vec![Spell::MagicMissile],
        )
    }
}

/// Sets up the multiplayer screen UI (connection phase).
///
/// If `PendingRematch` is present, skips the connection phase entirely and jumps
/// straight to wizard select, keeping the existing WebRTC connection alive.
pub fn setup(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    pending_rematch: Option<Res<PendingRematch>>,
) {
    // Rematch flow: skip connection phase, go straight to wizard select
    if pending_rematch.is_some() {
        commands.remove_resource::<PendingRematch>();

        // Clear any stale messages from the previous game
        connection.incoming_messages.clear();
        connection.outgoing_messages.clear();
        connection.incoming_unreliable.clear();
        connection.outgoing_unreliable.clear();

        let (my_wt, _my_sp) = load_my_unlocked_content();
        let initial_wizard = my_wt[0];

        commands.insert_resource(LobbyPhase::WizardSelect {
            my_wizard_types: my_wt.clone(),
            opponent_wizard_types: Vec::new(), // not needed for UI
            my_wizard: Some(initial_wizard),
            opponent_wizard: None,
            my_ready: false,
            opponent_ready: false,
        });

        // Notify opponent of our initial wizard selection
        connection
            .outgoing_messages
            .push(NetworkMessage::WizardSelected(initial_wizard));

        spawn_wizard_select_screen(&mut commands, &my_wt, initial_wizard, false, false);
        return;
    }

    // Normal connection flow
    connection.state = ConnectionState::Disconnected;
    connection.role = None;
    connection.local_code = None;
    connection.incoming_messages.clear();
    connection.outgoing_messages.clear();
    connection.incoming_unreliable.clear();
    connection.outgoing_unreliable.clear();
    connection.ping_ms = None;
    connection.ping_timer = 0.0;
    connection.error = None;

    #[cfg(target_arch = "wasm32")]
    crate::networking::webrtc::disconnect();

    commands.insert_resource(LobbyPhase::Connection);

    // Root container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(MARGIN),
                ..default()
            },
            OnMultiplayerScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Multiplayer"),
                TextFont {
                    font_size: MP_TITLE_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(MARGIN)),
                    ..default()
                },
            ));

            // Status text
            parent.spawn((
                StatusText,
                Text::new("Choose an option to get started"),
                TextFont {
                    font_size: STATUS_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));

            // Code display area (hidden initially)
            parent.spawn((
                CodeDisplayText,
                Text::new(""),
                TextFont {
                    font_size: CODE_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                TextLayout::new_with_linebreak(LineBreak::AnyCharacter),
                Node {
                    display: Display::None,
                    max_width: Val::Percent(80.0),
                    max_height: Val::Px(80.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
            ));

            // Ping display (hidden initially)
            parent.spawn((
                PingText,
                Text::new(""),
                TextFont {
                    font_size: STATUS_FONT_SIZE,
                    ..default()
                },
                TextColor(SUCCESS_COLOR),
                Node {
                    display: Display::None,
                    ..default()
                },
            ));

            // === Initial buttons (Host Game / Join Game) ===
            parent
                .spawn((
                    InitialButtons,
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(MARGIN * 0.5),
                        ..default()
                    },
                ))
                .with_children(|group| {
                    spawn_button(
                        group,
                        "Host Game",
                        MultiplayerButtonAction::HostGame,
                        &CONN_BUTTON_STYLE,
                    );
                    spawn_button(
                        group,
                        "Join Game",
                        MultiplayerButtonAction::JoinGame,
                        &CONN_BUTTON_STYLE,
                    );
                });

            // === Signaling buttons (Copy Code / Paste Response / Cancel) ===
            parent
                .spawn((
                    SignalingButtons,
                    Node {
                        display: Display::None,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(MARGIN * 0.5),
                        ..default()
                    },
                ))
                .with_children(|group| {
                    spawn_button(
                        group,
                        "Copy Code",
                        MultiplayerButtonAction::CopyCode,
                        &CONN_BUTTON_STYLE,
                    );
                    group
                        .spawn((
                            PasteResponseButton,
                            Node {
                                display: Display::None,
                                ..default()
                            },
                        ))
                        .with_children(|wrapper| {
                            spawn_button(
                                wrapper,
                                "Paste Response",
                                MultiplayerButtonAction::PasteResponse,
                                &CONN_BUTTON_STYLE,
                            );
                        });
                    spawn_button(
                        group,
                        "Cancel",
                        MultiplayerButtonAction::Disconnect,
                        &CONN_BUTTON_STYLE,
                    );
                });

            // === Active connection buttons (Try Again / Disconnect) ===
            parent
                .spawn((
                    ActiveConnectionButtons,
                    Node {
                        display: Display::None,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(MARGIN * 0.5),
                        ..default()
                    },
                ))
                .with_children(|group| {
                    spawn_button(
                        group,
                        "Try Again",
                        MultiplayerButtonAction::Retry,
                        &CONN_BUTTON_STYLE,
                    );
                    spawn_button(
                        group,
                        "Disconnect",
                        MultiplayerButtonAction::Disconnect,
                        &CONN_BUTTON_STYLE,
                    );
                });

            // Back button (always visible in connection phase)
            spawn_button(
                parent,
                "Back",
                MultiplayerButtonAction::Back,
                &CONN_BUTTON_STYLE,
            );
        });
}

/// Spawns the wizard select screen layout (mirrors single-player "Choose Your Path").
fn spawn_wizard_select_screen(
    commands: &mut Commands,
    unlocked_wizard_types: &[WizardType],
    initial_wizard: WizardType,
    opponent_ready: bool,
    my_ready: bool,
) {
    let wizard_types = WizardType::all();
    let unlocked_names: Vec<String> = unlocked_wizard_types
        .iter()
        .map(|wt| format!("{:?}", wt))
        .collect();

    commands.insert_resource(SelectedWizardPreview(initial_wizard));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(MARGIN * 1.5)),
                column_gap: Val::Px(MARGIN * 1.5),
                ..default()
            },
            OnMultiplayerScreen,
            WizardSelectScreen,
        ))
        .with_children(|root| {
            // ── Left panel ──────────────────────────────────────
            root.spawn(Node {
                width: Val::Px(LEFT_PANEL_WIDTH),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(MARGIN),
                ..default()
            })
            .with_children(|left| {
                // Top group: title + detail card
                left.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(MARGIN),
                    ..default()
                })
                .with_children(|top| {
                    shared::spawn_title_group(
                        top,
                        "Choose Your Path",
                        "Select your wizard for this match",
                    );

                    // Detail card (multiplayer-specific bottom section)
                    spawn_mp_detail_panel(
                        top,
                        initial_wizard,
                        opponent_ready,
                        my_ready,
                    );
                });

                // Bottom: disconnect button
                spawn_button(
                    left,
                    "Disconnect",
                    MultiplayerButtonAction::Disconnect,
                    &DISCONNECT_BUTTON_STYLE,
                );
            });

            // ── Right side: grid (reuses shared grid container + card helpers) ──
            root.spawn(grid_container_node())
                .with_children(|grid| {
                    for slot in 0..GRID_SLOTS {
                        if let Some(wizard_type) = wizard_types.get(slot) {
                            let type_name = format!("{:?}", wizard_type);
                            if unlocked_names.contains(&type_name) {
                                let is_selected = *wizard_type == initial_wizard;
                                shared::spawn_wizard_card(
                                    grid,
                                    *wizard_type,
                                    is_selected,
                                    MultiplayerButtonAction::PreviewWizard(*wizard_type),
                                );
                            } else {
                                shared::spawn_locked_wizard_card(grid, *wizard_type);
                            }
                        } else {
                            shared::spawn_locked_card(grid);
                        }
                    }
                });
        });
}

/// Spawns the multiplayer detail panel (shared top + MP-specific bottom with opponent info + Ready).
fn spawn_mp_detail_panel(
    parent: &mut ChildSpawnerCommands,
    wizard_type: WizardType,
    opponent_ready: bool,
    my_ready: bool,
) {
    shared::spawn_detail_panel_container(parent, |card| {
        shared::spawn_detail_panel_top(card, wizard_type);

        // Bottom: opponent status + ready/unready button (MP-specific)
        card.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            align_items: AlignItems::FlexStart,
            ..default()
        })
        .with_children(|bottom| {
            let status_text = build_opponent_status_text(opponent_ready);
            bottom.spawn((
                Text::new(status_text),
                TextFont {
                    font_size: DETAIL_STATUS_FONT_SIZE,
                    ..default()
                },
                TextColor(if opponent_ready {
                    SUCCESS_COLOR
                } else {
                    WAITING_COLOR
                }),
                DetailStatus,
            ));

            // Ready/Unready button area — rebuilt dynamically when ready state changes
            bottom
                .spawn((
                    ReadyButtonArea { showing_ready: my_ready },
                    Node::default(),
                ))
                .with_children(|area| {
                    spawn_ready_button_contents(area, my_ready);
                });
        });
    });
}

/// Spawns the contents of the ready button area (either Ready or Unready button).
fn spawn_ready_button_contents(parent: &mut ChildSpawnerCommands, my_ready: bool) {
    if my_ready {
        spawn_button(
            parent,
            "Unready",
            MultiplayerButtonAction::Unready,
            &UNREADY_BUTTON_STYLE,
        );
    } else {
        spawn_button(
            parent,
            "Ready!",
            MultiplayerButtonAction::Ready,
            &READY_BUTTON_STYLE,
        );
    }
}

/// Builds the opponent status display string.
fn build_opponent_status_text(opponent_ready: bool) -> String {
    if opponent_ready {
        "Opponent: Ready!".to_string()
    } else {
        "Opponent: choosing...".to_string()
    }
}

/// Cleans up the multiplayer screen UI when exiting the state.
pub fn cleanup(
    mut commands: Commands,
    screen_items: Query<Entity, With<OnMultiplayerScreen>>,
) {
    for entity in &screen_items {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<LobbyPhase>();
    commands.remove_resource::<SelectedWizardPreview>();
}

/// Handles multiplayer button actions.
#[allow(clippy::too_many_arguments)]
pub fn button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&MultiplayerButtonAction>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut connection: ResMut<NetworkConnection>,
    mut lobby_phase: ResMut<LobbyPhase>,
    mut preview: Option<ResMut<SelectedWizardPreview>>,
    mut detail_name: Query<
        &mut Text,
        (
            With<DetailName>,
            Without<DetailDescription>,
            Without<DetailStatus>,
        ),
    >,
    mut detail_desc: Query<
        &mut Text,
        (
            With<DetailDescription>,
            Without<DetailName>,
            Without<DetailStatus>,
        ),
    >,
    mut card_borders: Query<(&WizardCard, &mut BorderColor, &mut ButtonColors)>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                MultiplayerButtonAction::HostGame => {
                    connection.role = Some(PeerRole::Host);
                    connection.state = ConnectionState::WaitingForSignaling;
                    #[cfg(target_arch = "wasm32")]
                    crate::networking::webrtc::create_host_offer();
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        connection.state = ConnectionState::Failed;
                        connection.error =
                            Some("Multiplayer only available in browser".to_string());
                    }
                }
                MultiplayerButtonAction::JoinGame => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        if let Some(code) = crate::networking::clipboard::prompt_for_text(
                            "Paste the host's invite code:",
                        ) {
                            connection.role = Some(PeerRole::Guest);
                            connection.state = ConnectionState::WaitingForSignaling;
                            crate::networking::webrtc::create_guest_answer(&code);
                        }
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        connection.state = ConnectionState::Failed;
                        connection.error =
                            Some("Multiplayer only available in browser".to_string());
                    }
                }
                MultiplayerButtonAction::CopyCode => {
                    #[cfg(target_arch = "wasm32")]
                    if let Some(code) = &connection.local_code {
                        crate::networking::clipboard::copy_to_clipboard(code);
                    }
                }
                MultiplayerButtonAction::PasteResponse => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        if let Some(code) = crate::networking::clipboard::prompt_for_text(
                            "Paste the response code:",
                        ) {
                            crate::networking::webrtc::process_answer(&code);
                        }
                    }
                }
                MultiplayerButtonAction::PreviewWizard(wizard_type) => {
                    // Ignore wizard switches while readied — must unready first
                    if let LobbyPhase::WizardSelect { my_ready: true, .. } = &*lobby_phase {
                        continue;
                    }

                    if let Some(ref mut preview) = preview {
                        preview.0 = *wizard_type;
                    }

                    shared::update_detail_panel_text(
                        *wizard_type,
                        &mut detail_name,
                        &mut detail_desc,
                    );

                    // Update the selected wizard in lobby phase
                    if let LobbyPhase::WizardSelect { my_wizard, .. } = lobby_phase.as_mut() {
                        *my_wizard = Some(*wizard_type);
                        connection
                            .outgoing_messages
                            .push(NetworkMessage::WizardSelected(*wizard_type));
                    }

                    shared::update_card_borders(*wizard_type, &mut card_borders);
                }
                MultiplayerButtonAction::Ready => {
                    if let LobbyPhase::WizardSelect {
                        my_wizard: Some(_),
                        my_ready,
                        ..
                    } = lobby_phase.as_mut()
                    {
                        *my_ready = true;
                        connection
                            .outgoing_messages
                            .push(NetworkMessage::ReadyUp);
                    }
                }
                MultiplayerButtonAction::Unready => {
                    if let LobbyPhase::WizardSelect { my_ready, .. } = lobby_phase.as_mut() {
                        *my_ready = false;
                        connection
                            .outgoing_messages
                            .push(NetworkMessage::Unready);
                    }
                }
                MultiplayerButtonAction::Retry => {
                    let role = connection.role;
                    // Clean up old connection
                    #[cfg(target_arch = "wasm32")]
                    crate::networking::webrtc::disconnect();
                    connection.local_code = None;
                    connection.ping_ms = None;
                    connection.ping_timer = 0.0;
                    connection.error = None;
                    connection.incoming_messages.clear();
                    connection.outgoing_messages.clear();
                    connection.incoming_unreliable.clear();
                    connection.outgoing_unreliable.clear();

                    match role {
                        Some(PeerRole::Host) => {
                            connection.state = ConnectionState::WaitingForSignaling;
                            #[cfg(target_arch = "wasm32")]
                            crate::networking::webrtc::create_host_offer();
                        }
                        Some(PeerRole::Guest) => {
                            #[cfg(target_arch = "wasm32")]
                            {
                                if let Some(code) =
                                    crate::networking::clipboard::prompt_for_text(
                                        "Paste the host's invite code:",
                                    )
                                {
                                    connection.state = ConnectionState::WaitingForSignaling;
                                    crate::networking::webrtc::create_guest_answer(&code);
                                } else {
                                    // User cancelled the prompt, go back to disconnected
                                    connection.state = ConnectionState::Disconnected;
                                    connection.role = None;
                                }
                            }
                        }
                        None => {
                            connection.state = ConnectionState::Disconnected;
                        }
                    }
                }
                MultiplayerButtonAction::Disconnect | MultiplayerButtonAction::Back => {
                    #[cfg(target_arch = "wasm32")]
                    crate::networking::webrtc::disconnect();
                    connection.state = ConnectionState::Disconnected;
                    connection.role = None;
                    connection.local_code = None;
                    connection.ping_ms = None;
                    connection.error = None;
                    next_menu_state.set(MenuState::Landing);
                }
            }
        }
    }
}

/// Processes incoming network messages for the lobby (PlayerInfo exchange, wizard select, ready).
pub fn process_lobby_messages(
    connection: ResMut<NetworkConnection>,
    mut lobby_phase: ResMut<LobbyPhase>,
    mut commands: Commands,
    mut next_app_state: ResMut<NextState<AppState>>,
    screen_items: Query<Entity, With<OnMultiplayerScreen>>,
) {
    // Check if we need to send PlayerInfo (read-only check first to avoid triggering change detection)
    let should_send_info = connection.state == ConnectionState::Connected
        && *lobby_phase == LobbyPhase::Connection;

    // Check if there are messages to process (read-only)
    let has_messages = !connection.incoming_messages.is_empty();

    // Process network messages and PlayerInfo sending
    if should_send_info || has_messages {
        // Now take mutable access (triggers change detection, but only when we have real work)
        let mut connection = connection;

        // Send PlayerInfo when first connected
        if should_send_info {
            let (wizard_types, spells) = load_my_unlocked_content();

            info!("[Lobby] Connected! Sending PlayerInfo ({} wizard types, {} spells)", wizard_types.len(), spells.len());
            connection
                .outgoing_messages
                .push(NetworkMessage::PlayerInfo {
                    wizard_types: wizard_types.clone(),
                    spells: spells.clone(),
                });

            *lobby_phase = LobbyPhase::WaitingForPlayerInfo;
        }

        // Process incoming messages
        if has_messages {
            let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
            let mut unhandled = Vec::new();

            info!("[Lobby] Processing {} incoming messages", messages.len());
            for msg in messages {
                match msg {
                    NetworkMessage::PlayerInfo {
                        wizard_types: opponent_wt,
                        spells: _opponent_sp,
                    } => {
                        info!("[Lobby] Received PlayerInfo from opponent ({} wizard types)", opponent_wt.len());
                        let (my_wt, _my_sp) = load_my_unlocked_content();

                        // Despawn the connection-phase UI and spawn the wizard select screen
                        for entity in &screen_items {
                            commands.entity(entity).despawn();
                        }

                        let initial_wizard = my_wt[0];

                        *lobby_phase = LobbyPhase::WizardSelect {
                            my_wizard_types: my_wt.clone(),
                            opponent_wizard_types: opponent_wt,
                            my_wizard: Some(initial_wizard),
                            opponent_wizard: None,
                            my_ready: false,
                            opponent_ready: false,
                        };

                        // Notify opponent of our initial wizard selection
                        connection
                            .outgoing_messages
                            .push(NetworkMessage::WizardSelected(initial_wizard));

                        spawn_wizard_select_screen(
                            &mut commands,
                            &my_wt,
                            initial_wizard,
                            false,
                            false,
                        );
                    }
                    NetworkMessage::WizardSelected(wt) => {
                        if let LobbyPhase::WizardSelect {
                            opponent_wizard, ..
                        } = lobby_phase.as_mut()
                        {
                            *opponent_wizard = Some(wt);
                        }
                    }
                    NetworkMessage::ReadyUp => {
                        info!("[Lobby] Opponent readied up");
                        if let LobbyPhase::WizardSelect {
                            opponent_ready, ..
                        } = lobby_phase.as_mut()
                        {
                            *opponent_ready = true;
                        }
                    }
                    NetworkMessage::Unready => {
                        if let LobbyPhase::WizardSelect {
                            opponent_ready, ..
                        } = lobby_phase.as_mut()
                        {
                            *opponent_ready = false;
                        }
                    }
                    NetworkMessage::StartGame => {
                        info!("[Lobby] Received StartGame from host");
                        // Guest received StartGame from host
                        if let LobbyPhase::WizardSelect {
                            my_wizard: Some(my_wiz),
                            opponent_wizard: Some(opp_wiz),
                            ..
                        } = &*lobby_phase
                        {
                            let (_my_wt, my_spells) = load_my_unlocked_content();
                            let session = MultiplayerSession {
                                role: PeerRole::Guest,
                                host_wizard: *opp_wiz,
                                guest_wizard: *my_wiz,
                                host_spells: Vec::new(),
                                guest_spells: my_spells,
                            };
                            commands.insert_resource(session);
                        }
                        next_app_state.set(AppState::MultiplayerLoading);
                    }
                    other => {
                        unhandled.push(other);
                    }
                }
            }

            // Put unhandled messages back
            if !unhandled.is_empty() {
                connection.incoming_messages.extend(unhandled);
            }
        }

        // Check if both players are ready (host initiates)
        // This runs after processing messages so opponent's ReadyUp is handled first.
        check_both_ready(&lobby_phase, &mut connection, &mut commands, &mut next_app_state);
        return;
    }

    // No messages to process, but still check if both ready (handles the case
    // where the local player clicked Ready via button_action and opponent was
    // already ready from a previous frame).
    if lobby_phase.is_changed() {
        let mut connection = connection;
        check_both_ready(&lobby_phase, &mut connection, &mut commands, &mut next_app_state);
    }
}

/// Checks if both players are ready and initiates the game start (host only).
fn check_both_ready(
    lobby_phase: &LobbyPhase,
    connection: &mut NetworkConnection,
    commands: &mut Commands,
    next_app_state: &mut NextState<AppState>,
) {
    if let LobbyPhase::WizardSelect {
        my_wizard: Some(my_wiz),
        opponent_wizard: Some(opp_wiz),
        my_ready: true,
        opponent_ready: true,
        ..
    } = lobby_phase
        && connection.role == Some(PeerRole::Host)
    {
        info!("[Lobby] Both players ready! Host: {:?}, Guest: {:?}", my_wiz, opp_wiz);
        info!("[Lobby] I am host, sending StartGame and transitioning to MultiplayerLoading");
        let (_my_wt, my_spells) = load_my_unlocked_content();
        let session = MultiplayerSession {
            role: PeerRole::Host,
            host_wizard: *my_wiz,
            guest_wizard: *opp_wiz,
            host_spells: my_spells,
            guest_spells: Vec::new(),
        };
        commands.insert_resource(session);
        connection
            .outgoing_messages
            .push(NetworkMessage::StartGame);
        next_app_state.set(AppState::MultiplayerLoading);
    }
}

/// Updates the UI to reflect the current connection and lobby state.
///
/// In the connection phase, this toggles visibility of button groups and status text.
/// In the wizard select phase, it updates the detail panel status text.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_ui_state(
    mut commands: Commands,
    connection: Res<NetworkConnection>,
    lobby_phase: Res<LobbyPhase>,
    mut ready_button_area: Query<(Entity, &Children, &mut ReadyButtonArea)>,
    mut status_query: Query<
        (&mut Text, &mut TextColor),
        (With<StatusText>, Without<CodeDisplayText>, Without<PingText>),
    >,
    mut code_query: Query<
        (&mut Text, &mut Node),
        (
            With<CodeDisplayText>,
            Without<StatusText>,
            Without<PingText>,
            Without<InitialButtons>,
            Without<SignalingButtons>,
            Without<ActiveConnectionButtons>,
            Without<PasteResponseButton>,
        ),
    >,
    mut ping_query: Query<
        (&mut Text, &mut Node, &mut TextColor),
        (
            With<PingText>,
            Without<StatusText>,
            Without<CodeDisplayText>,
            Without<InitialButtons>,
            Without<SignalingButtons>,
            Without<ActiveConnectionButtons>,
            Without<PasteResponseButton>,
        ),
    >,
    mut initial_query: Query<
        &mut Node,
        (
            With<InitialButtons>,
            Without<SignalingButtons>,
            Without<ActiveConnectionButtons>,
            Without<CodeDisplayText>,
            Without<PingText>,
            Without<PasteResponseButton>,
        ),
    >,
    mut signaling_query: Query<
        &mut Node,
        (
            With<SignalingButtons>,
            Without<InitialButtons>,
            Without<ActiveConnectionButtons>,
            Without<CodeDisplayText>,
            Without<PingText>,
            Without<PasteResponseButton>,
        ),
    >,
    mut active_query: Query<
        &mut Node,
        (
            With<ActiveConnectionButtons>,
            Without<InitialButtons>,
            Without<SignalingButtons>,
            Without<CodeDisplayText>,
            Without<PingText>,
            Without<PasteResponseButton>,
        ),
    >,
    mut paste_query: Query<
        &mut Node,
        (
            With<PasteResponseButton>,
            Without<InitialButtons>,
            Without<SignalingButtons>,
            Without<ActiveConnectionButtons>,
            Without<CodeDisplayText>,
            Without<PingText>,
        ),
    >,
    // Wizard select phase queries
    mut detail_status: Query<
        (&mut Text, &mut TextColor),
        (
            With<DetailStatus>,
            Without<StatusText>,
            Without<CodeDisplayText>,
            Without<PingText>,
        ),
    >,
) {
    if !connection.is_changed() && !lobby_phase.is_changed() {
        return;
    }

    let in_wizard_select = matches!(&*lobby_phase, LobbyPhase::WizardSelect { .. });

    // In wizard select phase, update opponent status text + ready/unready button
    if in_wizard_select {
        if let LobbyPhase::WizardSelect {
            opponent_ready,
            my_ready,
            ..
        } = &*lobby_phase
        {
            if let Ok((mut text, mut color)) = detail_status.single_mut() {
                **text = build_opponent_status_text(*opponent_ready);
                if *opponent_ready {
                    color.0 = SUCCESS_COLOR;
                } else {
                    color.0 = WAITING_COLOR;
                }
            }

            // Rebuild the ready button area only when ready state actually changes
            if let Ok((entity, children, mut area)) = ready_button_area.single_mut()
                && area.showing_ready != *my_ready
            {
                area.showing_ready = *my_ready;
                for child in children.iter() {
                    commands.entity(child).despawn();
                }
                commands.entity(entity).with_children(|area| {
                    spawn_ready_button_contents(area, *my_ready);
                });
            }
        }
        return;
    }

    // Connection phase UI updates
    if let Ok((mut text, mut color)) = status_query.single_mut() {
        match connection.state {
            ConnectionState::Disconnected => {
                **text = "Choose an option to get started".to_string();
                color.0 = TEXT_COLOR;
            }
            ConnectionState::WaitingForSignaling => {
                if connection.local_code.is_some() {
                    match connection.role {
                        Some(PeerRole::Host) => {
                            **text =
                                "Code ready! Copy it and send to your friend.".to_string();
                        }
                        Some(PeerRole::Guest) => {
                            **text =
                                "Response ready! Copy it and send back to the host."
                                    .to_string();
                        }
                        None => {
                            **text = "Generating code...".to_string();
                        }
                    }
                } else {
                    **text = "Generating code...".to_string();
                }
                color.0 = WAITING_COLOR;
            }
            ConnectionState::Connecting => {
                **text = "Connecting...".to_string();
                color.0 = WAITING_COLOR;
            }
            ConnectionState::Connected => {
                **text = "Connected! Exchanging player info...".to_string();
                color.0 = SUCCESS_COLOR;
            }
            ConnectionState::Failed => {
                let msg = connection.error.as_deref().unwrap_or("Connection failed");
                **text = format!("Error: {}", msg);
                color.0 = ERROR_COLOR;
            }
        }
    }

    // Update code display
    if let Ok((mut text, mut node)) = code_query.single_mut() {
        if let Some(code) = &connection.local_code {
            **text = code.clone();
            node.display = Display::Flex;
        } else {
            **text = String::new();
            node.display = Display::None;
        }
    }

    // Update ping display
    if let Ok((mut text, mut node, mut color)) = ping_query.single_mut() {
        if let Some(ping) = connection.ping_ms {
            **text = format!("Ping: {:.0}ms", ping);
            node.display = Display::Flex;
            color.0 = SUCCESS_COLOR;
        } else {
            node.display = Display::None;
        }
    }

    // Toggle connection-phase button groups
    let show_initial = connection.state == ConnectionState::Disconnected;
    let show_signaling = connection.state == ConnectionState::WaitingForSignaling;
    let show_active = matches!(
        connection.state,
        ConnectionState::Connecting | ConnectionState::Connected | ConnectionState::Failed
    ) && !matches!(
        &*lobby_phase,
        LobbyPhase::WaitingForPlayerInfo | LobbyPhase::WizardSelect { .. }
    );

    if let Ok(mut node) = initial_query.single_mut() {
        node.display = if show_initial {
            Display::Flex
        } else {
            Display::None
        };
    }

    if let Ok(mut node) = signaling_query.single_mut() {
        node.display = if show_signaling {
            Display::Flex
        } else {
            Display::None
        };
    }

    if let Ok(mut node) = active_query.single_mut() {
        node.display = if show_active {
            Display::Flex
        } else {
            Display::None
        };
    }

    if let Ok(mut node) = paste_query.single_mut() {
        node.display = if show_signaling
            && connection.local_code.is_some()
            && connection.role == Some(PeerRole::Host)
        {
            Display::Flex
        } else {
            Display::None
        };
    }
}
