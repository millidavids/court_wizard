//! Multiplayer screen systems.

use bevy::prelude::*;

use crate::config::WizardType;
use crate::config::save_data;
use crate::game::input::messages::MouseClicked;
use crate::game::units::wizard::components::Spell;
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{ConnectionState, NetworkConnection, PeerRole};
use crate::networking::session::MultiplayerSession;
use crate::state::{AppState, MenuState};
use crate::ui::systems::spawn_button;

use super::components::{
    ActionBarContainer, ActiveConnectionButtons, CodeDisplayText, InitialButtons, LobbyPhase,
    LobbySpellSlot, LobbyWizardCard, MultiplayerButtonAction, OnMultiplayerScreen,
    OpponentInfoText, PasteResponseButton, PingText, ReadyButtonContainer, SignalingButtons,
    StatusText, WizardSelectContainer,
};
use super::constants::{
    BUTTON_STYLE, CARD_BG, CARD_BORDER, CARD_DESC_FONT_SIZE, CARD_NAME_FONT_SIZE,
    CARD_SELECTED_BORDER, CARD_SIZE, CODE_FONT_SIZE, ERROR_COLOR, MARGIN, READY_BUTTON_STYLE,
    SPELL_FONT_SIZE, SPELL_SLOT_BG, SPELL_SLOT_BORDER, SPELL_SLOT_EMPTY_BG, STATUS_FONT_SIZE,
    SUCCESS_COLOR, TEXT_COLOR, TITLE_FONT_SIZE, WAITING_COLOR,
};

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

/// Sets up the multiplayer screen UI.
///
/// Spawns all elements upfront. Visibility is toggled by `update_ui_state`.
pub fn setup(mut commands: Commands, mut connection: ResMut<NetworkConnection>) {
    // Reset connection state when entering the screen
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
                    font_size: TITLE_FONT_SIZE,
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
                        &BUTTON_STYLE,
                    );
                    spawn_button(
                        group,
                        "Join Game",
                        MultiplayerButtonAction::JoinGame,
                        &BUTTON_STYLE,
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
                        &BUTTON_STYLE,
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
                                &BUTTON_STYLE,
                            );
                        });
                    spawn_button(
                        group,
                        "Cancel",
                        MultiplayerButtonAction::Disconnect,
                        &BUTTON_STYLE,
                    );
                });

            // === Active connection buttons (Disconnect) ===
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
                        "Disconnect",
                        MultiplayerButtonAction::Disconnect,
                        &BUTTON_STYLE,
                    );
                });

            // === Wizard Select Container (hidden until wizard select phase) ===
            parent.spawn((
                WizardSelectContainer,
                Node {
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(MARGIN * 0.5),
                    ..default()
                },
            ));

            // === Opponent Info Text ===
            parent.spawn((
                OpponentInfoText,
                Text::new(""),
                TextFont {
                    font_size: STATUS_FONT_SIZE,
                    ..default()
                },
                TextColor(WAITING_COLOR),
                Node {
                    display: Display::None,
                    ..default()
                },
            ));

            // === Action Bar Container (hidden until wizard select phase) ===
            parent.spawn((
                ActionBarContainer,
                Node {
                    display: Display::None,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.0),
                    flex_wrap: FlexWrap::Wrap,
                    max_width: Val::Percent(80.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ));

            // === Ready Button Container ===
            parent.spawn((
                ReadyButtonContainer,
                Node {
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(MARGIN * 0.5),
                    ..default()
                },
            ));

            // Back button (always visible)
            spawn_button(
                parent,
                "Back",
                MultiplayerButtonAction::Back,
                &BUTTON_STYLE,
            );
        });
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
}

/// Handles multiplayer button actions.
pub fn button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&MultiplayerButtonAction>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut connection: ResMut<NetworkConnection>,
    mut lobby_phase: ResMut<LobbyPhase>,
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
                MultiplayerButtonAction::SelectWizard(wizard_type) => {
                    if let LobbyPhase::WizardSelect {
                        my_wizard,
                        my_action_bar,
                        my_ready,
                        ..
                    } = lobby_phase.as_mut()
                    {
                        *my_wizard = Some(*wizard_type);
                        *my_action_bar = [None; 5];
                        *my_ready = false;
                        connection
                            .outgoing_messages
                            .push(NetworkMessage::WizardSelected(*wizard_type));
                    }
                }
                MultiplayerButtonAction::ToggleSpell(spell) => {
                    if let LobbyPhase::WizardSelect {
                        my_action_bar,
                        my_ready,
                        ..
                    } = lobby_phase.as_mut()
                    {
                        let existing_slot = my_action_bar
                            .iter()
                            .position(|s| s.as_ref() == Some(spell));
                        if let Some(idx) = existing_slot {
                            my_action_bar[idx] = None;
                        } else if let Some(empty) =
                            my_action_bar.iter().position(|s| s.is_none())
                        {
                            my_action_bar[empty] = Some(*spell);
                        }
                        *my_ready = false;
                        connection
                            .outgoing_messages
                            .push(NetworkMessage::ActionBarSet(*my_action_bar));
                    }
                }
                MultiplayerButtonAction::Ready => {
                    if let LobbyPhase::WizardSelect {
                        my_wizard,
                        my_action_bar,
                        my_ready,
                        ..
                    } = lobby_phase.as_mut()
                    {
                        if my_wizard.is_some()
                            && my_action_bar.iter().any(|s| s.is_some())
                        {
                            *my_ready = true;
                            connection
                                .outgoing_messages
                                .push(NetworkMessage::ReadyUp);
                        }
                    }
                }
                MultiplayerButtonAction::Disconnect => {
                    #[cfg(target_arch = "wasm32")]
                    crate::networking::webrtc::disconnect();
                    connection.state = ConnectionState::Disconnected;
                    connection.role = None;
                    connection.local_code = None;
                    connection.ping_ms = None;
                    connection.error = None;
                    *lobby_phase = LobbyPhase::Connection;
                }
                MultiplayerButtonAction::Back => {
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
) {
    // Check if we need to send PlayerInfo (read-only check first to avoid triggering change detection)
    let should_send_info = connection.state == ConnectionState::Connected
        && *lobby_phase == LobbyPhase::Connection;

    // Check if there are messages to process (read-only)
    let has_messages = !connection.incoming_messages.is_empty();

    // Early return if nothing to do — avoids marking connection as changed
    if !should_send_info && !has_messages {
        return;
    }

    // Now take mutable access (triggers change detection, but only when we have real work)
    let mut connection = connection;

    // Send PlayerInfo when first connected
    if should_send_info {
        let (wizard_types, spells) = load_my_unlocked_content();

        connection
            .outgoing_messages
            .push(NetworkMessage::PlayerInfo {
                wizard_types: wizard_types.clone(),
                spells: spells.clone(),
            });

        *lobby_phase = LobbyPhase::WaitingForPlayerInfo;
        info!(
            "Sent PlayerInfo: {} wizard types, {} spells",
            wizard_types.len(),
            spells.len()
        );
    }

    // Process incoming messages
    if !has_messages {
        return;
    }
    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    for msg in messages {
        match msg {
            NetworkMessage::PlayerInfo {
                wizard_types: opponent_wt,
                spells: _opponent_sp,
            } => {
                let (my_wt, my_sp) = load_my_unlocked_content();
                *lobby_phase = LobbyPhase::WizardSelect {
                    my_wizard_types: my_wt,
                    my_spells: my_sp,
                    opponent_wizard_types: opponent_wt,
                    my_wizard: None,
                    opponent_wizard: None,
                    my_action_bar: [None; 5],
                    my_ready: false,
                    opponent_ready: false,
                };
            }
            NetworkMessage::WizardSelected(wt) => {
                if let LobbyPhase::WizardSelect {
                    opponent_wizard, ..
                } = lobby_phase.as_mut()
                {
                    *opponent_wizard = Some(wt);
                }
            }
            NetworkMessage::ActionBarSet(_) => {
                // We don't need to track opponent's action bar visually
            }
            NetworkMessage::ReadyUp => {
                if let LobbyPhase::WizardSelect {
                    opponent_ready, ..
                } = lobby_phase.as_mut()
                {
                    *opponent_ready = true;
                }
            }
            NetworkMessage::StartGame => {
                // Guest received StartGame from host
                if let LobbyPhase::WizardSelect {
                    my_wizard: Some(my_wiz),
                    opponent_wizard: Some(opp_wiz),
                    my_spells,
                    my_action_bar,
                    ..
                } = &*lobby_phase
                {
                    let session = MultiplayerSession {
                        role: PeerRole::Guest,
                        host_wizard: *opp_wiz,
                        guest_wizard: *my_wiz,
                        host_spells: Vec::new(),
                        guest_spells: my_spells.clone(),
                        host_action_bar: [None; 5],
                        guest_action_bar: *my_action_bar,
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

    // Check if both players are ready (host initiates)
    if let LobbyPhase::WizardSelect {
        my_wizard: Some(my_wiz),
        opponent_wizard: Some(opp_wiz),
        my_spells,
        my_action_bar,
        my_ready: true,
        opponent_ready: true,
        ..
    } = &*lobby_phase
    {
        if connection.role == Some(PeerRole::Host) {
            let session = MultiplayerSession {
                role: PeerRole::Host,
                host_wizard: *my_wiz,
                guest_wizard: *opp_wiz,
                host_spells: my_spells.clone(),
                guest_spells: Vec::new(),
                host_action_bar: *my_action_bar,
                guest_action_bar: [None; 5],
            };
            commands.insert_resource(session);
            connection
                .outgoing_messages
                .push(NetworkMessage::StartGame);
            next_app_state.set(AppState::MultiplayerLoading);
        }
    }
}

/// Updates the UI to reflect the current connection and lobby state.
pub fn update_ui_state(
    connection: Res<NetworkConnection>,
    lobby_phase: Res<LobbyPhase>,
    mut commands: Commands,
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
            Without<WizardSelectContainer>,
            Without<OpponentInfoText>,
            Without<ActionBarContainer>,
            Without<ReadyButtonContainer>,
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
            Without<WizardSelectContainer>,
            Without<OpponentInfoText>,
            Without<ActionBarContainer>,
            Without<ReadyButtonContainer>,
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
            Without<WizardSelectContainer>,
            Without<OpponentInfoText>,
            Without<ActionBarContainer>,
            Without<ReadyButtonContainer>,
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
            Without<WizardSelectContainer>,
            Without<OpponentInfoText>,
            Without<ActionBarContainer>,
            Without<ReadyButtonContainer>,
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
            Without<WizardSelectContainer>,
            Without<OpponentInfoText>,
            Without<ActionBarContainer>,
            Without<ReadyButtonContainer>,
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
            Without<WizardSelectContainer>,
            Without<OpponentInfoText>,
            Without<ActionBarContainer>,
            Without<ReadyButtonContainer>,
        ),
    >,
    mut wizard_select_query: Query<
        (Entity, &mut Node),
        (
            With<WizardSelectContainer>,
            Without<InitialButtons>,
            Without<SignalingButtons>,
            Without<ActiveConnectionButtons>,
            Without<CodeDisplayText>,
            Without<PingText>,
            Without<PasteResponseButton>,
            Without<OpponentInfoText>,
            Without<ActionBarContainer>,
            Without<ReadyButtonContainer>,
        ),
    >,
    mut opponent_info_query: Query<
        (&mut Text, &mut Node),
        (
            With<OpponentInfoText>,
            Without<StatusText>,
            Without<CodeDisplayText>,
            Without<PingText>,
            Without<InitialButtons>,
            Without<SignalingButtons>,
            Without<ActiveConnectionButtons>,
            Without<PasteResponseButton>,
            Without<WizardSelectContainer>,
            Without<ActionBarContainer>,
            Without<ReadyButtonContainer>,
        ),
    >,
    mut action_bar_query: Query<
        (Entity, &mut Node),
        (
            With<ActionBarContainer>,
            Without<InitialButtons>,
            Without<SignalingButtons>,
            Without<ActiveConnectionButtons>,
            Without<CodeDisplayText>,
            Without<PingText>,
            Without<PasteResponseButton>,
            Without<WizardSelectContainer>,
            Without<OpponentInfoText>,
            Without<ReadyButtonContainer>,
        ),
    >,
    mut ready_query: Query<
        (Entity, &mut Node),
        (
            With<ReadyButtonContainer>,
            Without<InitialButtons>,
            Without<SignalingButtons>,
            Without<ActiveConnectionButtons>,
            Without<CodeDisplayText>,
            Without<PingText>,
            Without<PasteResponseButton>,
            Without<WizardSelectContainer>,
            Without<OpponentInfoText>,
            Without<ActionBarContainer>,
        ),
    >,
) {
    if !connection.is_changed() && !lobby_phase.is_changed() {
        return;
    }

    let in_wizard_select = matches!(&*lobby_phase, LobbyPhase::WizardSelect { .. });

    // Update status text
    if let Ok((mut text, mut color)) = status_query.single_mut() {
        if in_wizard_select {
            if let LobbyPhase::WizardSelect {
                my_wizard,
                my_ready,
                opponent_ready,
                ..
            } = &*lobby_phase
            {
                if *my_ready && *opponent_ready {
                    **text = "Starting match...".to_string();
                    color.0 = SUCCESS_COLOR;
                } else if *my_ready {
                    **text = "Waiting for opponent...".to_string();
                    color.0 = WAITING_COLOR;
                } else if my_wizard.is_some() {
                    **text = "Select spells, then click Ready".to_string();
                    color.0 = TEXT_COLOR;
                } else {
                    **text = "Select your wizard".to_string();
                    color.0 = TEXT_COLOR;
                }
            }
        } else {
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
    }

    // Update code display
    if let Ok((mut text, mut node)) = code_query.single_mut() {
        if in_wizard_select {
            node.display = Display::None;
        } else if let Some(code) = &connection.local_code {
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
    let show_initial =
        connection.state == ConnectionState::Disconnected && !in_wizard_select;
    let show_signaling =
        connection.state == ConnectionState::WaitingForSignaling && !in_wizard_select;
    let show_active = !in_wizard_select
        && matches!(
            connection.state,
            ConnectionState::Connecting | ConnectionState::Connected | ConnectionState::Failed
        )
        && !matches!(
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

    // === Wizard Select Phase UI ===
    if let Ok((entity, mut node)) = wizard_select_query.single_mut() {
        if in_wizard_select {
            node.display = Display::Flex;

            if lobby_phase.is_changed() {
                // Despawn all children and rebuild
                commands.entity(entity).despawn_children();

                if let LobbyPhase::WizardSelect {
                    my_wizard_types,
                    my_wizard,
                    ..
                } = &*lobby_phase
                {
                    commands.entity(entity).with_children(|parent| {
                        parent
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|row| {
                                for wt in my_wizard_types {
                                    let is_selected = my_wizard.as_ref() == Some(wt);
                                    let border_color = if is_selected {
                                        CARD_SELECTED_BORDER
                                    } else {
                                        CARD_BORDER
                                    };
                                    row.spawn((
                                        Button,
                                        LobbyWizardCard(*wt),
                                        MultiplayerButtonAction::SelectWizard(*wt),
                                        Node {
                                            width: Val::Px(CARD_SIZE),
                                            height: Val::Px(CARD_SIZE),
                                            flex_direction: FlexDirection::Column,
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::Center,
                                            padding: UiRect::all(Val::Px(8.0)),
                                            border: UiRect::all(Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(CARD_BG),
                                        BorderColor::all(border_color),
                                        BorderRadius::all(Val::Px(6.0)),
                                    ))
                                    .with_children(|card| {
                                        card.spawn((
                                            Text::new(wt.display_name()),
                                            TextFont {
                                                font_size: CARD_NAME_FONT_SIZE,
                                                ..default()
                                            },
                                            TextColor(TEXT_COLOR),
                                        ));
                                        card.spawn((
                                            Text::new(wt.description()),
                                            TextFont {
                                                font_size: CARD_DESC_FONT_SIZE,
                                                ..default()
                                            },
                                            TextColor(WAITING_COLOR),
                                        ));
                                    });
                                }
                            });
                    });
                }
            }
        } else {
            node.display = Display::None;
        }
    }

    // === Opponent Info ===
    if let Ok((mut text, mut node)) = opponent_info_query.single_mut() {
        if let LobbyPhase::WizardSelect {
            opponent_wizard,
            opponent_ready,
            ..
        } = &*lobby_phase
        {
            node.display = Display::Flex;
            let opp_wiz_name = opponent_wizard
                .map(|w| w.display_name().to_string())
                .unwrap_or_else(|| "choosing...".to_string());
            let ready_str = if *opponent_ready { " (Ready!)" } else { "" };
            **text = format!("Opponent: {}{}", opp_wiz_name, ready_str);
        } else {
            node.display = Display::None;
        }
    }

    // === Action Bar / Spell Selection ===
    if let Ok((entity, mut node)) = action_bar_query.single_mut() {
        if let LobbyPhase::WizardSelect {
            my_wizard: Some(_),
            my_spells,
            my_action_bar,
            ..
        } = &*lobby_phase
        {
            node.display = Display::Flex;

            if lobby_phase.is_changed() {
                commands.entity(entity).despawn_children();

                commands.entity(entity).with_children(|parent| {
                    for spell in my_spells {
                        let is_equipped = my_action_bar.contains(&Some(*spell));
                        let bg = if is_equipped {
                            SPELL_SLOT_BG
                        } else {
                            SPELL_SLOT_EMPTY_BG
                        };
                        let border = if is_equipped {
                            SPELL_SLOT_BORDER
                        } else {
                            CARD_BORDER
                        };
                        parent
                            .spawn((
                                Button,
                                LobbySpellSlot(0),
                                MultiplayerButtonAction::ToggleSpell(*spell),
                                Node {
                                    width: Val::Px(70.0),
                                    height: Val::Px(40.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    padding: UiRect::horizontal(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(bg),
                                BorderColor::all(border),
                                BorderRadius::all(Val::Px(4.0)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new(spell.name()),
                                    TextFont {
                                        font_size: SPELL_FONT_SIZE,
                                        ..default()
                                    },
                                    TextColor(if is_equipped {
                                        TEXT_COLOR
                                    } else {
                                        WAITING_COLOR
                                    }),
                                ));
                            });
                    }
                });
            }
        } else {
            node.display = Display::None;
        }
    }

    // === Ready Button ===
    if let Ok((entity, mut node)) = ready_query.single_mut() {
        if let LobbyPhase::WizardSelect {
            my_wizard: Some(_),
            my_action_bar,
            my_ready,
            ..
        } = &*lobby_phase
        {
            let has_spells = my_action_bar.iter().any(|s| s.is_some());
            node.display = if has_spells { Display::Flex } else { Display::None };

            if lobby_phase.is_changed() {
                commands.entity(entity).despawn_children();
                commands.entity(entity).with_children(|parent| {
                    if !my_ready {
                        spawn_button(
                            parent,
                            "Ready!",
                            MultiplayerButtonAction::Ready,
                            &READY_BUTTON_STYLE,
                        );
                    }
                    spawn_button(
                        parent,
                        "Disconnect",
                        MultiplayerButtonAction::Disconnect,
                        &BUTTON_STYLE,
                    );
                });
            }
        } else {
            node.display = Display::None;
        }
    }
}
