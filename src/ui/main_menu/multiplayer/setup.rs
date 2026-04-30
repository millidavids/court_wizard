//! Multiplayer menu setup.

use bevy::prelude::*;

use crate::config::WizardType;
use crate::config::save_data;
use crate::game::multiplayer::components::PendingRematch;
use crate::game::units::wizard::components::Spell;
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{NetworkConnection, PeerRole};
use crate::networking::session::MultiplayerSession;
use crate::ui::systems::{spawn_button, spawn_page_container};

use super::components::{
    ActiveConnectionButtons, BackButton, CodeDisplayText, CopyCodeButton, InitialButtons,
    IpDisplayText, LanButtons, LanIpEntryButtons, LobbyPhase, MultiplayerButtonAction,
    OnMultiplayerScreen, PasteResponseButton, PingText, ReadyButtonArea, SignalingButtons,
    StatusText, TitleText, WizardSelectScreen,
};
use super::constants::*;
use super::shared::{self, DetailStatus, SelectedWizardPreview, grid_container_node};

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
pub(super) fn load_my_unlocked_content() -> (Vec<WizardType>, Vec<Spell>) {
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
pub(super) fn setup(
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
                        (
                            MultiplayerButtonAction::Back,
                            crate::ui::focus::NoGamepadFocus,
                        ),
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
pub(super) fn spawn_wizard_select_screen(
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
pub(super) fn spawn_ready_button_contents(parent: &mut ChildSpawnerCommands, my_ready: bool) {
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
pub(super) fn build_opponent_status_text(opponent_ready: bool) -> String {
    if opponent_ready {
        "Opponent: Ready!".to_string()
    } else {
        "Opponent: choosing...".to_string()
    }
}

/// Cleans up multiplayer-specific resources when exiting the state.
pub(super) fn cleanup_multiplayer_resources(mut commands: Commands) {
    commands.remove_resource::<LobbyPhase>();
    commands.remove_resource::<SelectedWizardPreview>();
}
