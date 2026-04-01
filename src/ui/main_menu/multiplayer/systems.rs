//! Multiplayer screen systems.

use bevy::prelude::*;

use crate::config::WizardType;
use crate::config::save_data;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::game::multiplayer::components::PendingRematch;
use crate::game::units::wizard::components::Spell;
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{ConnectionMode, ConnectionState, NetworkConnection, PeerRole};
use crate::networking::session::MultiplayerSession;
use crate::networking::transport::{TransportCommand, TransportHandle};
use crate::state::{AppState, MenuState};
use crate::ui::components::ButtonColors;
use crate::ui::systems::{spawn_button, spawn_page_container};

use super::super::wizard_select_shared::{self as shared};
use super::components::{
    ActiveConnectionButtons, BackButton, CodeDisplayText, CopyCodeButton, InitialButtons,
    IpDisplayText, LanButtons, LanIpEntryButtons, LobbyPhase, MultiplayerButtonAction,
    OnMultiplayerScreen, PasteResponseButton, PingText, ReadyButtonArea, SignalingButtons,
    StatusText, TitleText, WizardSelectScreen,
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

/// Loads this player's unlocked wizard types from save data.
///
/// All spells are available in multiplayer regardless of single-player progression.
fn load_my_unlocked_content() -> (Vec<WizardType>, Vec<Spell>) {
    let save = save_data::load_unified_save();
    let wt = if let Some(save) = &save {
        parse_wizard_types(&save.player.unlocked_content.wizard_types)
    } else {
        vec![WizardType::BoringOleMage]
    };
    let sp = Spell::all().to_vec();
    (wt, sp)
}

/// Sets up the multiplayer screen UI (connection phase).
///
/// If `PendingRematch` is present, skips the connection phase entirely and jumps
/// straight to wizard select, keeping the existing WebRTC connection alive.
pub fn setup(
    mut commands: Commands,
    mut connection: ResMut<NetworkConnection>,
    pending_rematch: Option<Res<PendingRematch>>,
    session: Option<Res<MultiplayerSession>>,
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

        // Pre-populate opponent's wizard from the previous game session
        // so both players can ready up without changing wizards
        let previous_opponent_wizard = session.as_ref().map(|s| match s.role {
            PeerRole::Host => s.guest_wizard,
            PeerRole::Guest => s.host_wizard,
        });

        commands.insert_resource(LobbyPhase::WizardSelect {
            my_wizard_types: my_wt.clone(),
            opponent_wizard_types: Vec::new(), // not needed for UI
            my_wizard: Some(initial_wizard),
            opponent_wizard: previous_opponent_wizard,
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

    connection.reset();

    commands.insert_resource(LobbyPhase::Connection);

    // Page container (standard overlay with content box)
    let content = spawn_page_container(
        &mut commands,
        OnMultiplayerScreen,
        false,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            padding: UiRect::all(Val::Px(MARGIN * 2.0)),
            column_gap: Val::Px(MARGIN * 2.0),
            border: UiRect::all(Val::Px(1.0)),
            overflow: Overflow::clip(),
            ..default()
        },
    );

    commands.entity(content).with_children(|root| {
        // ── Left column: buttons ──
        root.spawn(Node {
            width: Val::Px(CONN_LEFT_COLUMN_WIDTH),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(MARGIN * 0.5),
            ..default()
        })
        .with_children(|left| {
            // === Online section ===
            left.spawn((
                InitialButtons,
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: Val::Px(MARGIN * 0.5),
                    ..default()
                },
            ))
            .with_children(|group| {
                // Section label
                group.spawn((
                    Text::new("Online"),
                    TextFont::from_font_size(SECTION_LABEL_FONT_SIZE),
                    TextColor(SECTION_LABEL_COLOR),
                ));
                spawn_button(
                    group,
                    "Host Online Game",
                    MultiplayerButtonAction::HostGame,
                    &CONN_BUTTON_STYLE,
                );
                spawn_button(
                    group,
                    "Join Online Game",
                    MultiplayerButtonAction::JoinGame,
                    &CONN_BUTTON_STYLE,
                );
            });

            // === LAN section ===
            left.spawn((
                LanButtons,
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: Val::Px(MARGIN * 0.5),
                    margin: UiRect::top(Val::Px(MARGIN * 0.5)),
                    ..default()
                },
            ))
            .with_children(|group| {
                // Section label
                group.spawn((
                    Text::new("Local Network"),
                    TextFont::from_font_size(SECTION_LABEL_FONT_SIZE),
                    TextColor(SECTION_LABEL_COLOR),
                ));
                spawn_button(
                    group,
                    "Host LAN Game",
                    MultiplayerButtonAction::LanHost,
                    &CONN_BUTTON_STYLE,
                );
                spawn_button(
                    group,
                    "Join LAN Game",
                    MultiplayerButtonAction::LanJoin,
                    &CONN_BUTTON_STYLE,
                );
            });

            // === Signaling buttons (Cancel) ===
            left.spawn((
                SignalingButtons,
                Node {
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: Val::Px(MARGIN * 0.5),
                    ..default()
                },
            ))
            .with_children(|group| {
                spawn_button(
                    group,
                    "Cancel",
                    MultiplayerButtonAction::Cancel,
                    &CONN_BUTTON_STYLE,
                );
            });

            // === Active connection buttons (Try Again / Cancel) ===
            left.spawn((
                ActiveConnectionButtons,
                Node {
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
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
                    "Cancel",
                    MultiplayerButtonAction::Cancel,
                    &CONN_BUTTON_STYLE,
                );
            });

            // === LAN IP Entry buttons (hidden initially) ===
            left.spawn((
                LanIpEntryButtons,
                Node {
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: Val::Px(MARGIN * 0.5),
                    ..default()
                },
            ))
            .with_children(|group| {
                group.spawn((
                    Text::new("LAN Setup"),
                    TextFont::from_font_size(SECTION_LABEL_FONT_SIZE),
                    TextColor(SECTION_LABEL_COLOR),
                ));
                spawn_button(
                    group,
                    "Change IP",
                    MultiplayerButtonAction::LanEditIp,
                    &CONN_BUTTON_STYLE,
                );
                spawn_button(
                    group,
                    "Confirm & Connect",
                    MultiplayerButtonAction::LanConfirmIp,
                    &CONN_BUTTON_STYLE,
                );
                spawn_button(
                    group,
                    "Cancel",
                    MultiplayerButtonAction::LanIpCancel,
                    &CONN_BUTTON_STYLE,
                );
            });

            // Back button (visible on initial screen, hidden during connection)
            left.spawn((BackButton, Node::default()))
                .with_children(|wrapper| {
                    spawn_button(
                        wrapper,
                        "Back",
                        MultiplayerButtonAction::Back,
                        &CONN_BUTTON_STYLE,
                    );
                });
        });

        // ── Right column: info ──
        root.spawn(Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(MARGIN),
            justify_content: JustifyContent::FlexStart,
            ..default()
        })
        .with_children(|right| {
            // Title (dynamically updated to show mode/role)
            right.spawn((
                TitleText,
                Text::new("Multiplayer"),
                TextFont::from_font_size(MP_TITLE_FONT_SIZE),
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(MARGIN)),
                    ..default()
                },
            ));

            // Status text
            right.spawn((
                StatusText,
                Text::new("Choose an option to get started"),
                TextFont::from_font_size(STATUS_FONT_SIZE),
                TextColor(TEXT_COLOR),
            ));

            // IP display (hidden, shown during LAN IP entry)
            right.spawn((
                IpDisplayText,
                Text::new(""),
                TextFont::from_font_size(IP_DISPLAY_FONT_SIZE),
                TextColor(TEXT_COLOR),
                Node {
                    display: Display::None,
                    ..default()
                },
            ));

            // Time limit hint
            right.spawn((
                Text::new("Codes expire ~60 seconds after both are pasted — exchange quickly!"),
                TextFont::from_font_size(13.0),
                TextColor(Color::hsla(0.0, 0.0, 0.45, 1.0)),
            ));

            // Code display area (hidden initially)
            right.spawn((
                CodeDisplayText,
                Text::new(""),
                TextFont::from_font_size(CODE_FONT_SIZE),
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

            // Copy Code button (hidden initially, shown when code is ready)
            right
                .spawn((
                    CopyCodeButton,
                    Node {
                        display: Display::None,
                        ..default()
                    },
                ))
                .with_children(|wrapper| {
                    spawn_button(
                        wrapper,
                        "Copy Code",
                        MultiplayerButtonAction::CopyCode,
                        &CONN_BUTTON_STYLE,
                    );
                });

            // Paste Code button (hidden initially, shown for guest to paste host's code)
            right
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
                        "Paste Code",
                        MultiplayerButtonAction::PasteResponse,
                        &CONN_BUTTON_STYLE,
                    );
                });

            // Ping display (hidden initially)
            right.spawn((
                PingText,
                Text::new(""),
                TextFont::from_font_size(STATUS_FONT_SIZE),
                TextColor(SUCCESS_COLOR),
                Node {
                    display: Display::None,
                    ..default()
                },
            ));
        });
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

    let ws_content = spawn_page_container(
        commands,
        OnMultiplayerScreen,
        false,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            padding: UiRect::all(Val::Px(MARGIN * 1.5)),
            column_gap: Val::Px(MARGIN * 1.5),
            border: UiRect::all(Val::Px(1.0)),
            overflow: Overflow::clip(),
            ..default()
        },
    );
    commands.entity(ws_content).insert(WizardSelectScreen);

    commands.entity(ws_content).with_children(|root| {
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
                spawn_mp_detail_panel(top, initial_wizard, opponent_ready, my_ready);
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
        root.spawn(grid_container_node()).with_children(|grid| {
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
                TextFont::from_font_size(DETAIL_STATUS_FONT_SIZE),
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
                    ReadyButtonArea {
                        showing_ready: my_ready,
                    },
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

/// Cleans up multiplayer-specific resources when exiting the state.
pub fn cleanup_multiplayer_resources(mut commands: Commands) {
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
    transport: Option<Res<TransportHandle>>,
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
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    let send_cmd = |cmd: TransportCommand| {
        if let Some(ref transport) = transport {
            transport.send_command(cmd);
        }
    };

    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                MultiplayerButtonAction::HostGame => {
                    send_cmd(TransportCommand::CreateHost { use_relay: true });
                    connection.state = ConnectionState::WaitingForSignaling;
                    connection.role = Some(PeerRole::Host);
                    connection.mode = ConnectionMode::Online;
                }
                MultiplayerButtonAction::JoinGame => {
                    connection.state = ConnectionState::WaitingForSignaling;
                    connection.role = Some(PeerRole::Guest);
                    connection.mode = ConnectionMode::Online;
                }
                MultiplayerButtonAction::CopyCode => {
                    if let Some(code) = &connection.local_code
                        && let Ok(mut clipboard) = arboard::Clipboard::new()
                    {
                        let _ = clipboard.set_text(code.clone());
                    }
                }
                MultiplayerButtonAction::PasteResponse => {
                    match arboard::Clipboard::new().and_then(|mut cb| cb.get_text()) {
                        Ok(code) if !code.trim().is_empty() => {
                            send_cmd(TransportCommand::ConnectToHost { ticket_code: code });
                            connection.state = ConnectionState::Connecting;
                        }
                        _ => {
                            connection.error =
                                Some("Could not read code from clipboard".to_string());
                            connection.state = ConnectionState::Failed;
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
                        connection.outgoing_messages.push(NetworkMessage::ReadyUp);
                    }
                }
                MultiplayerButtonAction::Unready => {
                    if let LobbyPhase::WizardSelect { my_ready, .. } = lobby_phase.as_mut() {
                        *my_ready = false;
                        connection.outgoing_messages.push(NetworkMessage::Unready);
                    }
                }
                MultiplayerButtonAction::LanHost => {
                    send_cmd(TransportCommand::CreateHost { use_relay: false });
                    connection.state = ConnectionState::WaitingForSignaling;
                    connection.role = Some(PeerRole::Host);
                    connection.mode = ConnectionMode::Lan;
                }
                MultiplayerButtonAction::LanJoin => {
                    connection.state = ConnectionState::WaitingForSignaling;
                    connection.role = Some(PeerRole::Guest);
                    connection.mode = ConnectionMode::Lan;
                }
                MultiplayerButtonAction::LanEditIp => {
                    // Legacy LAN IP entry — no longer needed with ticket-based connection.
                }
                MultiplayerButtonAction::LanConfirmIp => {
                    // Legacy LAN IP confirm — no longer needed with ticket-based connection.
                }
                MultiplayerButtonAction::LanIpCancel => {
                    *lobby_phase = LobbyPhase::Connection;
                }
                MultiplayerButtonAction::Retry => {
                    let role = connection.role;
                    let is_lan = connection.mode == ConnectionMode::Lan;
                    send_cmd(TransportCommand::Disconnect);
                    connection.reset();

                    // Restore role/mode and re-enter the appropriate state.
                    connection.role = role;
                    connection.mode = if is_lan {
                        ConnectionMode::Lan
                    } else {
                        ConnectionMode::Online
                    };
                    match role {
                        Some(PeerRole::Host) => {
                            send_cmd(TransportCommand::CreateHost { use_relay: !is_lan });
                            connection.state = ConnectionState::WaitingForSignaling;
                        }
                        Some(PeerRole::Guest) => {
                            connection.state = ConnectionState::WaitingForSignaling;
                        }
                        None => {}
                    }
                }
                MultiplayerButtonAction::Cancel => {
                    send_cmd(TransportCommand::Disconnect);
                    connection.reset();
                }
                MultiplayerButtonAction::Disconnect => {
                    send_cmd(TransportCommand::Disconnect);
                    connection.reset();
                    channel_change.write(ChannelChangeMessage);
                    next_menu_state.set(MenuState::Landing);
                }
                MultiplayerButtonAction::Back => {
                    channel_change.write(ChannelChangeMessage);
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
    let should_send_info =
        connection.state == ConnectionState::Connected && *lobby_phase == LobbyPhase::Connection;

    // Check if there are messages to process (read-only)
    let has_messages = !connection.incoming_messages.is_empty();

    // Process network messages and PlayerInfo sending
    if should_send_info || has_messages {
        // Now take mutable access (triggers change detection, but only when we have real work)
        let mut connection = connection;

        // Send PlayerInfo when first connected
        if should_send_info {
            let (wizard_types, spells) = load_my_unlocked_content();

            info!(
                "[Lobby] Connected! Sending PlayerInfo ({} wizard types, {} spells)",
                wizard_types.len(),
                spells.len()
            );
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
                        info!(
                            "[Lobby] Received PlayerInfo from opponent ({} wizard types)",
                            opponent_wt.len()
                        );
                        let (my_wt, _my_sp) = load_my_unlocked_content();

                        // Despawn the connection-phase UI and spawn the wizard select screen
                        for entity in &screen_items {
                            commands.entity(entity).try_despawn();
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
                        if let LobbyPhase::WizardSelect { opponent_ready, .. } =
                            lobby_phase.as_mut()
                        {
                            *opponent_ready = true;
                        }
                    }
                    NetworkMessage::Unready => {
                        if let LobbyPhase::WizardSelect { opponent_ready, .. } =
                            lobby_phase.as_mut()
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
        check_both_ready(
            &lobby_phase,
            &mut connection,
            &mut commands,
            &mut next_app_state,
        );
        return;
    }

    // No messages to process, but still check if both ready (handles the case
    // where the local player clicked Ready via button_action and opponent was
    // already ready from a previous frame).
    if lobby_phase.is_changed() {
        let mut connection = connection;
        check_both_ready(
            &lobby_phase,
            &mut connection,
            &mut commands,
            &mut next_app_state,
        );
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
        info!(
            "[Lobby] Both players ready! Host: {:?}, Guest: {:?}",
            my_wiz, opp_wiz
        );
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
        connection.outgoing_messages.push(NetworkMessage::StartGame);
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
        (
            With<StatusText>,
            Without<CodeDisplayText>,
            Without<PingText>,
            Without<IpDisplayText>,
        ),
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
            Without<LanButtons>,
            Without<CopyCodeButton>,
            Without<BackButton>,
            Without<LanIpEntryButtons>,
            Without<IpDisplayText>,
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
            Without<LanButtons>,
            Without<CopyCodeButton>,
            Without<BackButton>,
            Without<LanIpEntryButtons>,
            Without<IpDisplayText>,
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
            Without<LanButtons>,
            Without<CopyCodeButton>,
            Without<BackButton>,
            Without<LanIpEntryButtons>,
            Without<IpDisplayText>,
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
            Without<LanButtons>,
            Without<CopyCodeButton>,
            Without<BackButton>,
            Without<LanIpEntryButtons>,
            Without<IpDisplayText>,
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
            Without<LanButtons>,
            Without<CopyCodeButton>,
            Without<BackButton>,
            Without<LanIpEntryButtons>,
            Without<IpDisplayText>,
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
            Without<LanButtons>,
            Without<CopyCodeButton>,
            Without<BackButton>,
            Without<LanIpEntryButtons>,
            Without<IpDisplayText>,
        ),
    >,
    mut lan_query: Query<
        &mut Node,
        (
            With<LanButtons>,
            Without<InitialButtons>,
            Without<SignalingButtons>,
            Without<ActiveConnectionButtons>,
            Without<CodeDisplayText>,
            Without<PingText>,
            Without<PasteResponseButton>,
            Without<CopyCodeButton>,
            Without<BackButton>,
            Without<LanIpEntryButtons>,
            Without<IpDisplayText>,
        ),
    >,
    mut copy_code_query: Query<
        &mut Node,
        (
            With<CopyCodeButton>,
            Without<InitialButtons>,
            Without<SignalingButtons>,
            Without<ActiveConnectionButtons>,
            Without<CodeDisplayText>,
            Without<PingText>,
            Without<PasteResponseButton>,
            Without<LanButtons>,
            Without<BackButton>,
            Without<LanIpEntryButtons>,
            Without<IpDisplayText>,
        ),
    >,
    mut back_query: Query<
        &mut Node,
        (
            With<BackButton>,
            Without<InitialButtons>,
            Without<SignalingButtons>,
            Without<ActiveConnectionButtons>,
            Without<CodeDisplayText>,
            Without<PingText>,
            Without<PasteResponseButton>,
            Without<LanButtons>,
            Without<CopyCodeButton>,
            Without<LanIpEntryButtons>,
            Without<IpDisplayText>,
        ),
    >,
    // Bundle these as a nested tuple to stay within Bevy's 16-param limit
    (mut lan_ip_entry_query, mut ip_display_query, mut detail_status, mut title_query): (
        Query<
            &mut Node,
            (
                With<LanIpEntryButtons>,
                Without<InitialButtons>,
                Without<SignalingButtons>,
                Without<ActiveConnectionButtons>,
                Without<CodeDisplayText>,
                Without<PingText>,
                Without<PasteResponseButton>,
                Without<LanButtons>,
                Without<CopyCodeButton>,
                Without<BackButton>,
                Without<IpDisplayText>,
            ),
        >,
        Query<
            (&mut Text, &mut Node),
            (
                With<IpDisplayText>,
                Without<StatusText>,
                Without<CodeDisplayText>,
                Without<PingText>,
                Without<InitialButtons>,
                Without<SignalingButtons>,
                Without<ActiveConnectionButtons>,
                Without<PasteResponseButton>,
                Without<LanButtons>,
                Without<CopyCodeButton>,
                Without<BackButton>,
                Without<LanIpEntryButtons>,
                Without<TitleText>,
            ),
        >,
        Query<
            (&mut Text, &mut TextColor),
            (
                With<DetailStatus>,
                Without<StatusText>,
                Without<CodeDisplayText>,
                Without<PingText>,
                Without<IpDisplayText>,
                Without<TitleText>,
            ),
        >,
        Query<
            &mut Text,
            (
                With<TitleText>,
                Without<StatusText>,
                Without<CodeDisplayText>,
                Without<PingText>,
                Without<IpDisplayText>,
                Without<DetailStatus>,
            ),
        >,
    ),
) {
    if !connection.is_changed() && !lobby_phase.is_changed() {
        return;
    }

    let in_wizard_select = matches!(&*lobby_phase, LobbyPhase::WizardSelect { .. });
    let in_lan_ip_entry = matches!(&*lobby_phase, LobbyPhase::LanIpEntry { .. });

    // Update title to reflect current mode and role
    if let Ok(mut title) = title_query.single_mut() {
        let mode_str = match connection.mode {
            ConnectionMode::Online => "Online",
            ConnectionMode::Lan => "LAN",
        };
        let role_str = match connection.role {
            Some(PeerRole::Host) => "Host",
            Some(PeerRole::Guest) => "Guest",
            None => "",
        };

        // Determine title based on current phase
        if in_lan_ip_entry {
            if let LobbyPhase::LanIpEntry { role, .. } = &*lobby_phase {
                let lan_role = match role {
                    PeerRole::Host => "Host",
                    PeerRole::Guest => "Guest",
                };
                **title = format!("Multiplayer - LAN - {}", lan_role);
            }
        } else if connection.state == ConnectionState::Disconnected && connection.role.is_none() {
            **title = "Multiplayer".to_string();
        } else {
            **title = format!("Multiplayer - {} - {}", mode_str, role_str);
        }
    }

    // LAN IP entry phase: update IP display and status text
    if in_lan_ip_entry {
        if let LobbyPhase::LanIpEntry { role, current_ip } = &*lobby_phase {
            let role_str = match role {
                PeerRole::Host => "Host",
                PeerRole::Guest => "Join",
            };
            if let Ok((mut text, mut node)) = ip_display_query.single_mut() {
                if let Some(ip) = current_ip {
                    **text = format!("LAN {} — Your IP: {}", role_str, ip);
                } else {
                    **text = format!("LAN {} — No IP set", role_str);
                }
                node.display = Display::Flex;
            }
            if let Ok((mut text, mut color)) = status_query.single_mut() {
                if current_ip.is_some() {
                    **text = "IP configured. Click 'Confirm & Connect' to proceed.".to_string();
                } else {
                    **text = "Click 'Change IP' to enter your local IP address.".to_string();
                }
                color.0 = TEXT_COLOR;
            }
        }
    } else if let Ok((_, mut node)) = ip_display_query.single_mut() {
        node.display = Display::None;
    }

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
                    commands.entity(child).try_despawn();
                }
                commands.entity(entity).with_children(|area| {
                    spawn_ready_button_contents(area, *my_ready);
                });
            }
        }
        return;
    }

    // Connection phase UI updates (skip if in LAN IP entry — that phase sets its own status)
    if !in_lan_ip_entry && let Ok((mut text, mut color)) = status_query.single_mut() {
        match connection.state {
            ConnectionState::Disconnected => {
                **text = "Choose an option to get started".to_string();
                color.0 = TEXT_COLOR;
            }
            ConnectionState::WaitingForSignaling => {
                match connection.role {
                    Some(PeerRole::Host) => {
                        if connection.local_code.is_some() {
                            **text = "Code ready! Copy it and send to your friend. Waiting for them to connect...".to_string();
                        } else {
                            **text = "Generating connection code...".to_string();
                        }
                    }
                    Some(PeerRole::Guest) => {
                        **text = "Copy the host's code to your clipboard, then click 'Paste Code'."
                            .to_string();
                    }
                    None => {
                        **text = "Generating code...".to_string();
                    }
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
    let show_initial = connection.state == ConnectionState::Disconnected && !in_lan_ip_entry;
    let show_signaling =
        connection.state == ConnectionState::WaitingForSignaling && !in_lan_ip_entry;
    let show_active = matches!(
        connection.state,
        ConnectionState::Connecting | ConnectionState::Connected | ConnectionState::Failed
    ) && !matches!(
        &*lobby_phase,
        LobbyPhase::WaitingForPlayerInfo
            | LobbyPhase::WizardSelect { .. }
            | LobbyPhase::LanIpEntry { .. }
    );

    if let Ok(mut node) = initial_query.single_mut() {
        node.display = if show_initial {
            Display::Flex
        } else {
            Display::None
        };
    }

    // LAN buttons follow the same visibility as initial buttons
    for mut node in &mut lan_query {
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
        // Show paste button for guest to paste the host's connection code.
        let show_paste = show_signaling && connection.role == Some(PeerRole::Guest);
        node.display = if show_paste {
            Display::Flex
        } else {
            Display::None
        };
    }

    // Show Copy Code button in the right column when code is ready
    if let Ok(mut node) = copy_code_query.single_mut() {
        let show_copy = show_signaling && connection.local_code.is_some();
        node.display = if show_copy {
            Display::Flex
        } else {
            Display::None
        };
    }

    // LAN IP entry buttons
    if let Ok(mut node) = lan_ip_entry_query.single_mut() {
        node.display = if in_lan_ip_entry {
            Display::Flex
        } else {
            Display::None
        };
    }

    // Back button only visible on the initial screen (hidden during signaling/active)
    if let Ok(mut node) = back_query.single_mut() {
        node.display = if show_initial {
            Display::Flex
        } else {
            Display::None
        };
    }
}
