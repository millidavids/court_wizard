use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use rand::Rng;

use crate::config::save_data;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::game_mode::components::{ActiveToggles, RogueliteModifiers, ToggleModifier};
use crate::game::input::messages::MouseClicked;
use crate::state::MenuState;
use crate::ui::constants::{SLIDER_TRACK_WIDTH, TEXT_DISABLED, TEXT_MUTED};
use crate::ui::systems::{
    SliderRowConfig, spawn_button, spawn_page_container,
    spawn_slider_row,
};

use super::components::{
    ConfirmUnlockAction, ConfirmUnlockPopup, ExpandedToggles, ModifierButtonAction,
    ModifierSliderDownButton, ModifierSliderFill, ModifierSliderHandle, ModifierSliderText,
    ModifierSliderTrack, ModifierSliderUpButton, ModifierSliderValue, OnRogueliteModifiersScreen,
    PendingToggles, RunSummaryContent, ScrollableModifierList, ScrollableRunSummary, SeedInputBox,
    SeedInputState, SeedInputText, SeedRandomButton, ToggleDescriptionNode, ToggleExpandButton,
    ToggleRowContainer, ToggleUnlockButton,
};
use super::constants::*;

/// Maximum seed value (10 digits, fits in the text box).
const MAX_SEED: u64 = 10_000_000_000;
/// Maximum number of characters in the seed input field.
const MAX_SEED_CHARS: usize = 10;

fn random_seed() -> u64 {
    rand::thread_rng().gen_range(0..MAX_SEED)
}

// ── Setup ──────────────────────────────────────────────────────────────────

/// Sets up the roguelite modifiers screen UI with a two-panel layout.
pub(super) fn setup(
    mut commands: Commands,
    modifiers: Option<Res<RogueliteModifiers>>,
    existing_pending: Option<Res<PendingToggles>>,
    active_toggles: Option<Res<ActiveToggles>>,
    mut config: ResMut<crate::config::GameConfig>,
) {
    let already_exists = modifiers.is_some();
    let mods = modifiers.map(|m| m.clone()).unwrap_or_default();
    if !already_exists {
        commands.insert_resource(mods.clone());
    }

    // Only preserve pending toggles if ActiveToggles exists (returning from wizard select).
    // Otherwise start fresh — prevents stale selections from previous runs.
    let pending_toggles = if active_toggles.is_some() {
        existing_pending
            .map(|p| PendingToggles {
                enabled: p.enabled.clone(),
            })
            .unwrap_or_default()
    } else {
        PendingToggles::default()
    };
    commands.insert_resource(pending_toggles.clone());

    // Always generate a fresh random seed when opening this page
    let seed = random_seed();
    config.seed = Some(seed);
    let seed_text = seed.to_string();
    commands.insert_resource(SeedInputState {
        text: seed_text.clone(),
        focused: false,
    });

    commands.insert_resource(ExpandedToggles::default());

    // Column root: title + two-panel row (matches compendium pattern)
    let content = spawn_page_container(
        &mut commands,
        OnRogueliteModifiersScreen,
        false,
        crate::ui::systems::default_content_node(),
    );

    commands.entity(content).with_children(|root| {
        // Title
        crate::ui::systems::spawn_title_with_shadow(
            root,
            "Run Modifiers",
            20.0,
            TEXT_COLOR,
            Node {
                margin: UiRect::bottom(Val::Px(MARGIN_SMALL)),
                ..default()
            },
        );

        // Two-panel row — fills remaining height after title
        root.spawn(Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            flex_basis: Val::Px(0.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(crate::ui::constants::TWO_PANEL_GAP),
            ..default()
        })
        .with_children(|row| {
            spawn_left_panel(row, &mods, &pending_toggles);
            spawn_right_panel(row, &mods, &pending_toggles, &seed_text);
        });
    });
}

// ── Left Panel (Run Summary) ───────────────────────────────────────────────

/// Spawns the left panel showing a summary of active modifiers and navigation buttons.
fn spawn_left_panel(
    parent: &mut ChildSpawnerCommands,
    mods: &RogueliteModifiers,
    pending_toggles: &PendingToggles,
) {
    let detail_box =
        crate::ui::systems::spawn_scrollable_left_detail_panel(parent, ScrollableRunSummary);

    // Add content to the detail box
    parent.commands().entity(detail_box).with_children(|panel| {
        // Title
        panel.spawn((
            Text::new("Your Run"),
            TextFont::from_font_size(SUMMARY_TITLE_FONT_SIZE),
            TextColor(TEXT_COLOR),
        ));

        // Summary content container (rebuilt dynamically)
        panel
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                RunSummaryContent,
            ))
            .with_children(|summary| {
                spawn_summary_items(summary, mods, pending_toggles);
            });
    });

    // Buttons go below the detail box (inside the left panel outer container).
    // The detail box's parent is the left panel outer node.
    parent.commands().entity(detail_box).with_children(|panel| {
        // Spacer to push buttons to bottom
        panel.spawn(Node {
            flex_grow: 1.0,
            ..default()
        });

        panel
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|buttons| {
                spawn_button(
                    buttons,
                    "Start Run",
                    ModifierButtonAction::StartRun,
                    &START_RUN_BUTTON_STYLE,
                );
                spawn_button(
                    buttons,
                    "Back",
                    ModifierButtonAction::Back,
                    &BACK_BUTTON_STYLE,
                );
            });
    });
}

/// Spawns text items inside the run summary content container.
fn spawn_summary_items(
    parent: &mut ChildSpawnerCommands,
    mods: &RogueliteModifiers,
    pending_toggles: &PendingToggles,
) {
    let mut has_items = false;

    // Slider modifiers at non-default values
    for (label, pct) in mods.non_default_entries() {
        parent.spawn((
            Text::new(format!("{}: {}%", label, pct)),
            TextFont::from_font_size(SUMMARY_ITEM_FONT_SIZE),
            TextColor(SUMMARY_ITEM_COLOR),
        ));
        has_items = true;
    }

    // Enabled toggle names
    for toggle in &pending_toggles.enabled {
        parent.spawn((
            Text::new(toggle.display_name()),
            TextFont::from_font_size(SUMMARY_ITEM_FONT_SIZE),
            TextColor(SUMMARY_ITEM_COLOR),
        ));
        has_items = true;
    }

    // Placeholder if nothing active
    if !has_items {
        parent.spawn((
            Text::new("Default settings"),
            TextFont::from_font_size(SUMMARY_ITEM_FONT_SIZE),
            TextColor(SUMMARY_PLACEHOLDER_COLOR),
        ));
    }
}

// ── Right Panel (Configuration) ────────────────────────────────────────────

/// Spawns the right panel with seed input, sliders, and toggle modifiers.
/// Uses the compendium pattern: outer Column + nested scrollable container.
fn spawn_right_panel(
    parent: &mut ChildSpawnerCommands,
    mods: &RogueliteModifiers,
    pending_toggles: &PendingToggles,
    seed_text: &str,
) {
    // Outer column wrapper (takes remaining width)
    parent
        .spawn(Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|right_outer| {
            // Scrollable content area (matches compendium scroll pattern)
            right_outer
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        flex_basis: Val::Px(0.0),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    ScrollPosition::default(),
                    ScrollableModifierList,
                    BackgroundColor(crate::ui::constants::LIST_BG),
                    BorderColor::all(crate::ui::constants::LIST_BORDER),
                    BorderRadius::all(Val::Px(crate::ui::constants::PANEL_BORDER_RADIUS)),
                ))
                .with_children(|scroll| {
                    // Inner content column
                    scroll
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(MARGIN_SMALL),
                            padding: UiRect::all(Val::Px(16.0)),
                            ..default()
                        })
                        .with_children(|right| {
                            // Seed input row
                            spawn_seed_input_row(right, seed_text);

                            // Sliders
                            let sliders = [
                                ModifierSliderValue::GameSpeed,
                                ModifierSliderValue::EnemyEffectiveness,
                                ModifierSliderValue::EnemyCount,
                                ModifierSliderValue::TerrainDensity,
                            ];
                            for slider_value in sliders {
                                spawn_modifier_slider(right, slider_value, mods);
                            }

                            // Toggle Modifiers section header
                            right.spawn((
                                Text::new("Toggle Modifiers"),
                                TextFont::from_font_size(SECTION_HEADER_FONT_SIZE),
                                TextColor(TEXT_COLOR),
                                Node {
                                    margin: UiRect::new(
                                        Val::ZERO,
                                        Val::ZERO,
                                        Val::Px(MARGIN),
                                        Val::Px(MARGIN_SMALL),
                                    ),
                                    ..default()
                                },
                            ));

                            // Toggle modifier rows
                            let unlocked_ids = save_data::get_unlocked_toggles();
                            for &toggle in ToggleModifier::all() {
                                let is_unlocked =
                                    unlocked_ids.iter().any(|id| id == toggle.id());
                                let is_enabled =
                                    is_unlocked && pending_toggles.is_enabled(toggle);
                                spawn_toggle_row(right, toggle, is_unlocked, is_enabled);
                            }
                        });
                });
        });
}

/// Spawns the seed input row with label, text input box, and Randomize button.
fn spawn_seed_input_row(parent: &mut ChildSpawnerCommands, seed_text: &str) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(crate::ui::constants::SLIDER_GAP),
            margin: UiRect::bottom(Val::Px(MARGIN)),
            ..default()
        })
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new("Seed"),
                TextFont::from_font_size(crate::ui::constants::SLIDER_LABEL_FONT_SIZE),
                TextColor(TEXT_COLOR),
                Node {
                    min_width: Val::Px(200.0),
                    width: Val::Px(200.0),
                    ..default()
                },
            ));

            // Input box (clickable)
            row.spawn((
                Button,
                Node {
                    width: Val::Px(280.0),
                    height: Val::Px(32.0),
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::horizontal(Val::Px(8.0)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(Color::hsla(270.0, 0.35, 0.35, 1.0)),
                BorderRadius::all(Val::Px(4.0)),
                BackgroundColor(Color::hsla(270.0, 0.08, 0.08, 1.0)),
                crate::ui::components::ButtonColors {
                    background: Color::hsla(270.0, 0.08, 0.08, 1.0),
                    border: Color::hsla(270.0, 0.35, 0.35, 1.0),
                },
                SeedInputBox,
            ))
            .with_children(|input_box| {
                input_box.spawn((
                    Text::new(seed_text),
                    TextFont::from_font_size(14.0),
                    TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
                    SeedInputText,
                ));
            });

            // Random button
            row.spawn((
                Button,
                Node {
                    height: Val::Px(32.0),
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::horizontal(Val::Px(12.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(Color::hsla(270.0, 0.35, 0.35, 1.0)),
                BorderRadius::all(Val::Px(4.0)),
                BackgroundColor(Color::hsla(270.0, 0.08, 0.10, 1.0)),
                crate::ui::components::ButtonColors {
                    background: Color::hsla(270.0, 0.08, 0.10, 1.0),
                    border: Color::hsla(270.0, 0.35, 0.35, 1.0),
                },
                SeedRandomButton,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Randomize"),
                    TextFont::from_font_size(12.0),
                    TextColor(TEXT_COLOR),
                ));
            });
        });
}

/// Spawns a single modifier slider using the shared slider row helper.
fn spawn_modifier_slider(
    parent: &mut ChildSpawnerCommands,
    slider_value: ModifierSliderValue,
    modifiers: &RogueliteModifiers,
) {
    let current_value = slider_value.get(modifiers);

    spawn_slider_row(
        parent,
        SliderRowConfig {
            label: slider_value.label(),
            current_value,
            min_value: slider_value.min_value(),
            max_value: slider_value.max_value(),
            label_width: 200.0,
            text_component: ModifierSliderText {
                value: slider_value,
            },
            down_button: ModifierSliderDownButton {
                value: slider_value,
            },
            up_button: ModifierSliderUpButton {
                value: slider_value,
            },
            slider_track: ModifierSliderTrack {
                value: slider_value,
            },
            slider_fill: ModifierSliderFill {
                value: slider_value,
            },
            slider_handle: ModifierSliderHandle {
                value: slider_value,
                is_dragging: false,
            },
        },
    );
}

// ── Toggle Rows ────────────────────────────────────────────────────────────

/// Spawns a single toggle modifier row.
/// - Unlocked: entire row is clickable to toggle on/off, turns purple when active.
/// - Locked: shows Insight cost centered, clicking row opens unlock popup.
fn spawn_toggle_row(
    parent: &mut ChildSpawnerCommands,
    toggle: ToggleModifier,
    is_unlocked: bool,
    is_enabled: bool,
) {
    let (bg, border) = if !is_unlocked {
        (TOGGLE_LOCKED_BG, TOGGLE_LOCKED_BORDER)
    } else if is_enabled {
        (TOGGLE_ON_BG, TOGGLE_ON_BORDER)
    } else {
        (TOGGLE_OFF_BG, TOGGLE_OFF_BORDER)
    };

    // Outer row: [toggle button (grows) | expand arrow button]
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                column_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_children(|row| {
            // Toggle button (takes remaining space)
            row.spawn((
                Button,
                Node {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(6.0)),
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                BackgroundColor(bg),
                BorderColor::all(border),
                BorderRadius::all(Val::Px(6.0)),
                crate::ui::components::ButtonColors {
                    background: bg,
                    border,
                },
                ToggleRowContainer(toggle),
            ))
            .insert_if(crate::ui::components::ButtonActive, || is_enabled)
            .with_children(|toggle_btn| {
                // Header content: [Name ... (Insight cost if locked)]
                toggle_btn
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|header| {
                        let name_color = if is_unlocked {
                            TEXT_COLOR
                        } else {
                            TEXT_DISABLED
                        };
                        header.spawn((
                            Text::new(toggle.display_name()),
                            TextFont::from_font_size(TOGGLE_NAME_FONT_SIZE),
                            TextColor(name_color),
                            Node {
                                flex_grow: 1.0,
                                ..default()
                            },
                        ));

                        if !is_unlocked {
                            header.spawn((
                                Text::new(format!("{} Insight", toggle.insight_cost())),
                                TextFont::from_font_size(TOGGLE_SMALL_BUTTON_FONT_SIZE),
                                TextColor(crate::ui::constants::INSIGHT_COLOR),
                                ToggleUnlockButton(toggle),
                            ));
                        }
                    });

                // Description (hidden by default, expands the toggle button)
                toggle_btn.spawn((
                    Text::new(toggle.description()),
                    TextFont::from_font_size(TOGGLE_DESC_FONT_SIZE),
                    TextColor(DESCRIPTION_COLOR),
                    Node {
                        display: Display::None,
                        max_width: Val::Percent(95.0),
                        padding: UiRect::left(Val::Px(4.0)),
                        ..default()
                    },
                    ToggleDescriptionNode(toggle),
                ));
            });

            // Expand arrow button (separate, to the right)
            row.spawn((
                Button,
                Node {
                    width: Val::Px(TOGGLE_SMALL_BUTTON_SIZE),
                    height: Val::Px(TOGGLE_SMALL_BUTTON_SIZE),
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(border),
                BorderRadius::all(Val::Px(4.0)),
                BackgroundColor(bg),
                crate::ui::components::ButtonColors {
                    background: bg,
                    border,
                },
                ToggleExpandButton(toggle),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("\u{25b8}"),
                    TextFont::from_font_size(TOGGLE_SMALL_BUTTON_FONT_SIZE),
                    TextColor(TEXT_MUTED),
                ));
            });
        });
}

// ── Confirmation Popup ─────────────────────────────────────────────────────

/// Spawns a confirmation popup for unlocking a toggle modifier.
fn spawn_unlock_popup(commands: &mut Commands, toggle: ToggleModifier) {
    let current_insight = save_data::get_insight();
    let cost = toggle.insight_cost();
    let can_afford = current_insight >= cost;

    let message = if can_afford {
        format!(
            "Unlock \"{}\" for {} Insight?\n(You have {} Insight)",
            toggle.display_name(),
            cost,
            current_insight
        )
    } else {
        format!(
            "Not enough Insight to unlock \"{}\".\nCost: {} | You have: {}",
            toggle.display_name(),
            cost,
            current_insight
        )
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(POPUP_OVERLAY_BG),
            GlobalZIndex(600),
            ConfirmUnlockPopup,
            OnRogueliteModifiersScreen,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(30.0)),
                        row_gap: Val::Px(20.0),
                        border: UiRect::all(Val::Px(2.0)),
                        min_width: Val::Px(350.0),
                        ..default()
                    },
                    BackgroundColor(POPUP_BOX_BG),
                    BorderColor::all(POPUP_BOX_BORDER),
                    BorderRadius::all(Val::Px(8.0)),
                ))
                .with_children(|popup| {
                    popup.spawn((
                        Text::new(message),
                        TextFont::from_font_size(POPUP_FONT_SIZE),
                        TextColor(TEXT_COLOR),
                        TextLayout::new_with_justify(Justify::Center),
                    ));

                    popup
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(16.0),
                            ..default()
                        })
                        .with_children(|buttons| {
                            if can_afford {
                                spawn_button(
                                    buttons,
                                    "Confirm",
                                    ConfirmUnlockAction::Confirm(toggle),
                                    &CONFIRM_BUTTON_STYLE,
                                );
                            }
                            spawn_button(
                                buttons,
                                "Cancel",
                                ConfirmUnlockAction::Cancel,
                                &CANCEL_BUTTON_STYLE,
                            );
                        });
                });
        });
}

// ── Button Systems ─────────────────────────────────────────────────────────

/// Handles Back and Start Run button actions.
pub(super) fn button_action(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&ModifierButtonAction>,
    mut next_state: ResMut<NextState<MenuState>>,
    pending_toggles: Res<PendingToggles>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                ModifierButtonAction::StartRun => {
                    channel_change.write(ChannelChangeMessage);
                    commands.insert_resource(ActiveToggles::new(
                        pending_toggles.enabled.clone(),
                    ));
                    next_state.set(MenuState::WizardSelect);
                }
                ModifierButtonAction::Back => {
                    channel_change.write(ChannelChangeMessage);
                    next_state.set(MenuState::GameModeSelect);
                }
            }
        }
    }
}

/// Handles Escape key to go back to game mode select.
pub(super) fn escape_to_game_mode_select(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        channel_change.write(ChannelChangeMessage);
        next_state.set(MenuState::GameModeSelect);
    }
}

// ── Toggle Systems ─────────────────────────────────────────────────────────

/// Toggles the expand/collapse state of a toggle modifier's description.
pub(super) fn toggle_expand_action(
    mut button_clicked: MessageReader<MouseClicked>,
    expand_buttons: Query<&ToggleExpandButton>,
    mut expanded: ResMut<ExpandedToggles>,
    mut descriptions: Query<(&ToggleDescriptionNode, &mut Node)>,
    children_query: Query<&Children>,
    mut text_query: Query<&mut Text>,
    expand_btn_entities: Query<(Entity, &ToggleExpandButton)>,
) {
    for event in button_clicked.read() {
        let Ok(btn) = expand_buttons.get(event.button) else {
            continue;
        };
        let toggle = btn.0;

        let is_expanded = expanded.0.contains(&toggle);
        if is_expanded {
            expanded.0.remove(&toggle);
        } else {
            expanded.0.insert(toggle);
        }
        let now_expanded = !is_expanded;

        // Update description visibility
        for (desc, mut node) in &mut descriptions {
            if desc.0 == toggle {
                node.display = if now_expanded {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }

        // Update arrow text
        for (entity, expand_btn) in &expand_btn_entities {
            if expand_btn.0 == toggle {
                if let Ok(children) = children_query.get(entity) {
                    for child in children.iter() {
                        if let Ok(mut text) = text_query.get_mut(child) {
                            text.0 = if now_expanded {
                                "\u{25be}".to_string()
                            } else {
                                "\u{25b8}".to_string()
                            };
                        }
                    }
                }
            }
        }
    }
}

/// Toggles ON/OFF for unlocked toggle modifiers by clicking the row itself.
/// For locked toggles, clicking the row opens the unlock confirmation popup.
pub(super) fn toggle_row_action(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    row_query: Query<&ToggleRowContainer>,
    expand_buttons: Query<&ToggleExpandButton>,
    mut pending_toggles: ResMut<PendingToggles>,
    mut row_containers: Query<(
        Entity,
        &ToggleRowContainer,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut crate::ui::components::ButtonColors,
    )>,
    existing_popup: Query<Entity, With<ConfirmUnlockPopup>>,
) {
    for event in button_clicked.read() {
        // Skip if the click was on the expand button (handled separately)
        if expand_buttons.get(event.button).is_ok() {
            continue;
        }

        let Ok(row) = row_query.get(event.button) else {
            continue;
        };
        let toggle = row.0;
        let is_unlocked = save_data::is_toggle_unlocked(toggle);

        if !is_unlocked {
            // Locked — open unlock confirmation popup
            if existing_popup.is_empty() {
                spawn_unlock_popup(&mut commands, toggle);
            }
            continue;
        }

        // Unlocked — toggle on/off
        pending_toggles.toggle(toggle);
        let now_enabled = pending_toggles.is_enabled(toggle);

        let (bg, border) = if now_enabled {
            (TOGGLE_ON_BG, TOGGLE_ON_BORDER)
        } else {
            (TOGGLE_OFF_BG, TOGGLE_OFF_BORDER)
        };

        for (entity, container, mut bg_color, mut border_color, mut btn_colors) in &mut row_containers {
            if container.0 == toggle {
                bg_color.0 = bg;
                *border_color = BorderColor::all(border);
                btn_colors.background = bg;
                btn_colors.border = border;

                if now_enabled {
                    commands.entity(entity).insert(crate::ui::components::ButtonActive);
                } else {
                    commands.entity(entity).remove::<crate::ui::components::ButtonActive>();
                }
            }
        }
    }
}



/// Handles confirm/cancel actions in the unlock confirmation popup.
pub(super) fn handle_unlock_confirmation(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    action_query: Query<&ConfirmUnlockAction>,
    popup_query: Query<Entity, With<ConfirmUnlockPopup>>,
    mut pending_toggles: ResMut<PendingToggles>,
    mut row_containers: Query<(
        Entity,
        &ToggleRowContainer,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut crate::ui::components::ButtonColors,
    )>,
    unlock_cost_texts: Query<(Entity, &ToggleUnlockButton)>,
) {
    for event in button_clicked.read() {
        let Ok(action) = action_query.get(event.button) else {
            continue;
        };

        match action {
            ConfirmUnlockAction::Confirm(toggle) => {
                let toggle = *toggle;
                if save_data::unlock_toggle(toggle) {
                    pending_toggles.enabled.push(toggle);

                    // Update row visual to enabled state
                    for (entity, container, mut bg_color, mut border_color, mut btn_colors) in
                        &mut row_containers
                    {
                        if container.0 == toggle {
                            bg_color.0 = TOGGLE_ON_BG;
                            *border_color = BorderColor::all(TOGGLE_ON_BORDER);
                            commands.entity(entity).insert(crate::ui::components::ButtonActive);
                            btn_colors.background = TOGGLE_ON_BG;
                            btn_colors.border = TOGGLE_ON_BORDER;
                        }
                    }

                    // Remove the Insight cost text
                    for (entity, unlock_btn) in &unlock_cost_texts {
                        if unlock_btn.0 == toggle {
                            commands.entity(entity).try_despawn();
                            break;
                        }
                    }
                }
            }
            ConfirmUnlockAction::Cancel => {}
        }

        // Dismiss popup
        for entity in &popup_query {
            commands.entity(entity).try_despawn();
        }
    }
}

// ── Run Summary Update ─────────────────────────────────────────────────────

/// Rebuilds the left panel summary text when modifiers or pending toggles change.
pub(super) fn update_run_summary(
    mut commands: Commands,
    modifiers: Res<RogueliteModifiers>,
    pending_toggles: Res<PendingToggles>,
    summary_query: Query<Entity, With<RunSummaryContent>>,
    children_query: Query<&Children>,
) {
    if !modifiers.is_changed() && !pending_toggles.is_changed() {
        return;
    }

    let Ok(summary_entity) = summary_query.single() else {
        return;
    };

    // Despawn all existing children
    if let Ok(children) = children_query.get(summary_entity) {
        for child in children.iter() {
            commands.entity(child).try_despawn();
        }
    }

    // Re-spawn fresh text nodes
    commands.entity(summary_entity).with_children(|summary| {
        spawn_summary_items(summary, &modifiers, &pending_toggles);
    });
}

// ── Slider Systems ─────────────────────────────────────────────────────────

/// Handles slider +/- button clicks.
pub(super) fn slider_button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    down_buttons: Query<&ModifierSliderDownButton>,
    up_buttons: Query<&ModifierSliderUpButton>,
    mut modifiers: ResMut<RogueliteModifiers>,
) {
    for event in button_clicked.read() {
        if let Ok(button) = down_buttons.get(event.button) {
            let current = button.value.get(&modifiers);
            let step = button.value.step();
            let min = button.value.min_value();
            let new_value = (current - step).max(min);
            button.value.set(&mut modifiers, new_value);
        } else if let Ok(button) = up_buttons.get(event.button) {
            let current = button.value.get(&modifiers);
            let step = button.value.step();
            let max = button.value.max_value();
            let new_value = (current + step).min(max);
            button.value.set(&mut modifiers, new_value);
        }
    }
}

/// Handles dragging slider handles and clicking on tracks.
pub(super) fn slider_interaction(
    buttons: Res<ButtonInput<bevy::input::mouse::MouseButton>>,
    mut slider_handles: Query<(&Interaction, &mut ModifierSliderHandle)>,
    slider_tracks: Query<(&Interaction, &RelativeCursorPosition, &ModifierSliderTrack)>,
    mut modifiers: ResMut<RogueliteModifiers>,
) {
    // Stop dragging when mouse is released
    if !buttons.pressed(bevy::input::mouse::MouseButton::Left) {
        for (_interaction, mut slider_handle) in &mut slider_handles {
            slider_handle.is_dragging = false;
        }
        return;
    }

    // Check if track was clicked (start dragging)
    if buttons.just_pressed(bevy::input::mouse::MouseButton::Left) {
        for (interaction, _cursor_pos, track) in &slider_tracks {
            if matches!(*interaction, Interaction::Pressed | Interaction::Hovered) {
                // Start dragging the corresponding handle
                for (_handle_interaction, mut slider_handle) in &mut slider_handles {
                    if slider_handle.value == track.value {
                        slider_handle.is_dragging = true;
                    }
                }
            }
        }

        // Also start dragging if the handle itself was clicked
        for (interaction, mut slider_handle) in &mut slider_handles {
            if *interaction == Interaction::Pressed {
                slider_handle.is_dragging = true;
            }
        }
    }

    // While dragging, use the track's RelativeCursorPosition to set the value.
    // This gives pixel-perfect tracking regardless of scale factor or viewport.
    for (_interaction, cursor_pos, track) in &slider_tracks {
        let is_dragging = slider_handles
            .iter()
            .any(|(_, h)| h.value == track.value && h.is_dragging);

        if is_dragging && let Some(pos) = cursor_pos.normalized {
            // RelativeCursorPosition.normalized: center at (0,0),
            // left edge = -0.5, right edge = 0.5
            let normalized = (pos.x + 0.5).clamp(0.0, 1.0);

            let min = track.value.min_value();
            let max = track.value.max_value();
            let range = max - min;
            let new_value = (min + normalized * range).clamp(min, max);

            // Snap to nearest step
            let step = track.value.step();
            let snapped = (new_value / step).round() * step;
            let snapped = snapped.clamp(min, max);

            if (track.value.get(&modifiers) - snapped).abs() > f32::EPSILON {
                track.value.set(&mut modifiers, snapped);
            }
        }
    }
}

/// Updates slider text displays when modifiers change.
pub(super) fn update_slider_text(
    modifiers: Res<RogueliteModifiers>,
    mut slider_texts: Query<(&mut Text, &ModifierSliderText)>,
) {
    if modifiers.is_changed() {
        for (mut text, slider_text) in &mut slider_texts {
            let value = slider_text.value.get(&modifiers);
            text.0 = format!("{}%", (value * 100.0) as u32);
        }
    }
}

/// Updates slider fill widths and handle positions when modifiers change.
pub(super) fn update_sliders(
    modifiers: Res<RogueliteModifiers>,
    mut slider_fills: Query<(&mut Node, &ModifierSliderFill), Without<ModifierSliderHandle>>,
    mut slider_handles: Query<(&mut Node, &ModifierSliderHandle), Without<ModifierSliderFill>>,
) {
    if modifiers.is_changed() {
        for (mut node, slider_fill) in &mut slider_fills {
            let value = slider_fill.value.get(&modifiers);
            let min = slider_fill.value.min_value();
            let max = slider_fill.value.max_value();
            let range = max - min;
            let normalized = (value - min) / range;
            node.width = Val::Percent(normalized * 100.0);
        }

        for (mut node, slider_handle) in &mut slider_handles {
            let value = slider_handle.value.get(&modifiers);
            let min = slider_handle.value.min_value();
            let max = slider_handle.value.max_value();
            let range = max - min;
            let normalized = (value - min) / range;
            node.left = Val::Px(normalized * SLIDER_TRACK_WIDTH - 2.0);
        }
    }
}

// ── Seed Input Systems ─────────────────────────────────────────────────────

/// Handles clicking the seed input box to focus it, or clicking Random / other buttons to unfocus.
pub(super) fn seed_input_click(
    mut button_clicked: MessageReader<MouseClicked>,
    input_boxes: Query<Entity, With<SeedInputBox>>,
    random_buttons: Query<Entity, With<SeedRandomButton>>,
    mut seed_state: ResMut<SeedInputState>,
    mut config: ResMut<crate::config::GameConfig>,
    mut text_query: Query<&mut Text, With<SeedInputText>>,
    mut border_query: Query<&mut BorderColor, With<SeedInputBox>>,
) {
    for event in button_clicked.read() {
        if input_boxes.get(event.button).is_ok() {
            // Clicked the input box — toggle focus
            seed_state.focused = !seed_state.focused;
            // When focusing, clear the text so user can type fresh
            if seed_state.focused {
                seed_state.text.clear();
                for mut text in &mut text_query {
                    text.0 = String::new();
                }
            }
        } else if random_buttons.get(event.button).is_ok() {
            // Clicked "Random" — generate a new random seed
            let new_seed = random_seed();
            seed_state.text = new_seed.to_string();
            seed_state.focused = false;
            config.seed = Some(new_seed);
            for mut text in &mut text_query {
                text.0 = new_seed.to_string();
            }
        } else {
            // Clicked something else — unfocus
            if seed_state.focused {
                seed_state.focused = false;
                // If the user cleared the field, restore with a random seed
                if seed_state.text.is_empty() {
                    let new_seed = random_seed();
                    seed_state.text = new_seed.to_string();
                    config.seed = Some(new_seed);
                    for mut text in &mut text_query {
                        text.0 = new_seed.to_string();
                    }
                }
            }
        }
    }

    // Update border color to indicate focus
    for mut border in &mut border_query {
        if seed_state.focused {
            *border = BorderColor::all(Color::hsla(270.0, 0.65, 0.55, 1.0));
        } else {
            *border = BorderColor::all(Color::hsla(270.0, 0.35, 0.35, 1.0));
        }
    }
}

/// Handles keyboard input when the seed field is focused.
pub(super) fn seed_input_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut seed_state: ResMut<SeedInputState>,
    mut config: ResMut<crate::config::GameConfig>,
    mut text_query: Query<&mut Text, With<SeedInputText>>,
    mut border_query: Query<&mut BorderColor, With<SeedInputBox>>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    // Ctrl+C: Copy seed to clipboard (works even when not focused)
    if ctrl
        && keyboard.just_pressed(KeyCode::KeyC)
        && !seed_state.text.is_empty()
        && let Ok(mut clipboard) = arboard::Clipboard::new()
    {
        let _ = clipboard.set_text(seed_state.text.clone());
    }

    if !seed_state.focused {
        return;
    }

    let mut changed = false;

    // Ctrl+V: Paste seed from clipboard
    if ctrl
        && keyboard.just_pressed(KeyCode::KeyV)
        && let Ok(text) = arboard::Clipboard::new().and_then(|mut cb| cb.get_text())
    {
        // Extract only digits, limit to 10 characters
        let digits: String = text
            .chars()
            .filter(|c| c.is_ascii_digit())
            .take(MAX_SEED_CHARS)
            .collect();
        if !digits.is_empty() {
            seed_state.text = digits;
            changed = true;
        }
    }

    // Number keys (main keyboard)
    for (key, digit) in [
        (KeyCode::Digit0, '0'),
        (KeyCode::Digit1, '1'),
        (KeyCode::Digit2, '2'),
        (KeyCode::Digit3, '3'),
        (KeyCode::Digit4, '4'),
        (KeyCode::Digit5, '5'),
        (KeyCode::Digit6, '6'),
        (KeyCode::Digit7, '7'),
        (KeyCode::Digit8, '8'),
        (KeyCode::Digit9, '9'),
    ] {
        if keyboard.just_pressed(key) && seed_state.text.len() < MAX_SEED_CHARS {
            seed_state.text.push(digit);
            changed = true;
        }
    }

    // Numpad keys
    for (key, digit) in [
        (KeyCode::Numpad0, '0'),
        (KeyCode::Numpad1, '1'),
        (KeyCode::Numpad2, '2'),
        (KeyCode::Numpad3, '3'),
        (KeyCode::Numpad4, '4'),
        (KeyCode::Numpad5, '5'),
        (KeyCode::Numpad6, '6'),
        (KeyCode::Numpad7, '7'),
        (KeyCode::Numpad8, '8'),
        (KeyCode::Numpad9, '9'),
    ] {
        if keyboard.just_pressed(key) && seed_state.text.len() < MAX_SEED_CHARS {
            seed_state.text.push(digit);
            changed = true;
        }
    }

    // Backspace
    if keyboard.just_pressed(KeyCode::Backspace) {
        seed_state.text.pop();
        changed = true;
    }

    // Enter/Escape to unfocus
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter) {
        seed_state.focused = false;
        // If empty after typing, generate random
        if seed_state.text.is_empty() {
            let new_seed = random_seed();
            seed_state.text = new_seed.to_string();
            config.seed = Some(new_seed);
            for mut text in &mut text_query {
                text.0 = new_seed.to_string();
            }
        }
        for mut border in &mut border_query {
            *border = BorderColor::all(Color::hsla(270.0, 0.35, 0.35, 1.0));
        }
        return;
    }

    if changed {
        // Update config seed
        config.seed = seed_state.text.parse::<u64>().ok();

        // Update display text
        for mut text in &mut text_query {
            text.0 = seed_state.text.clone();
        }
    }
}
