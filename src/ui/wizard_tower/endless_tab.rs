use bevy::prelude::*;

use crate::config::save_data::{EndlessLevelBest, load_unified_save};
use crate::config::{GameConfig, WizardType};
use crate::game::constants::boss_name_for_level;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::game_mode::components::{GameMode, format_time};
use crate::game::input::messages::MouseClicked;
use crate::game::resources::{CurrentLevel, KillStats, TimeTravelState};
use crate::state::AppState;
use crate::ui::systems::spawn_button;

use super::components::*;
use super::constants::*;
use super::layout::RightPanelView;

// ---------------------------------------------------------------------------
// Action enum
// ---------------------------------------------------------------------------

/// Actions for buttons within the endless tab.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum EndlessAction {
    ContinuePlay,
    SwitchWizardType,
    StartTimeTravel,
}

// ---------------------------------------------------------------------------
// Right panel builder
// ---------------------------------------------------------------------------

/// Populates the right panel with endless mode content:
/// - "Switch Wizard Type" button
/// - "Continue Playing" button
/// - Time Travel section (if player has beaten at least level 1)
pub(super) fn build_endless_right_panel(
    commands: &mut Commands,
    right_panel_entity: Entity,
    config: &GameConfig,
) {
    let show_time_travel = config.highest_level_achieved > 1;

    if show_time_travel {
        init_time_travel_resource(commands);
    }

    commands
        .entity(right_panel_entity)
        .with_children(|wrapper| {
            // Two-column layout
            wrapper
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    padding: UiRect::all(Val::Px(SECTION_PADDING)),
                    column_gap: Val::Px(20.0),
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|row| {
                    // Left column: title + buttons
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        width: Val::Percent(40.0),
                        ..default()
                    })
                    .with_children(|left_col| {
                        // Buttons stacked vertically
                        spawn_button(
                            left_col,
                            "Continue Playing",
                            EndlessAction::ContinuePlay,
                            &BUTTON_STYLE,
                        );

                        spawn_button(
                            left_col,
                            "Switch Wizard",
                            EndlessAction::SwitchWizardType,
                            &SWITCH_WIZARD_BUTTON_STYLE,
                        );
                    });

                    // Right column: Time Travel
                    if show_time_travel {
                        row.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            flex_grow: 1.0,
                            ..default()
                        })
                        .with_children(|right_col| {
                            spawn_time_travel_section(right_col, config);
                        });
                    }
                });
        });
}

// ---------------------------------------------------------------------------
// Left panel builder
// ---------------------------------------------------------------------------

/// Populates the left panel with the selected wizard's type, level, and endless progress stats.
pub(super) fn build_endless_left_panel(
    commands: &mut Commands,
    left_panel_entity: Entity,
    wizard_type: WizardType,
    config: &GameConfig,
) {
    let save = load_unified_save();

    commands.entity(left_panel_entity).with_children(|parent| {
        // Wizard name
        parent.spawn((
            Text::new(wizard_type.display_name()),
            TextFont::from_font_size(DETAIL_TITLE_FONT_SIZE),
            TextColor(TITLE_COLOR),
        ));

        // Wizard description
        parent.spawn((
            Text::new(wizard_type.description()),
            TextFont::from_font_size(DETAIL_TEXT_FONT_SIZE),
            TextColor(TEXT_COLOR),
        ));

        // Current level
        parent.spawn((
            Text::new(format!("Level {}", config.current_level)),
            TextFont::from_font_size(LEVEL_FONT_SIZE),
            TextColor(AFFINITY_COLOR),
            LevelDisplay,
            Node {
                margin: UiRect::bottom(Val::Px(MARGIN_SMALL)),
                ..default()
            },
        ));

        // Endless progress stats
        parent.spawn((
            Text::new("Endless Progress"),
            TextFont::from_font_size(STAT_SECTION_FONT_SIZE),
            TextColor(STAT_SECTION_COLOR),
            Node {
                margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(10.0), Val::Px(2.0)),
                ..default()
            },
        ));

        // Aggregate best stats for this wizard type
        let all_levels = aggregate_endless_stats(&save, wizard_type);

        if all_levels.is_empty() {
            parent.spawn((
                Text::new("No data yet."),
                TextFont::from_font_size(DETAIL_TEXT_FONT_SIZE),
                TextColor(LOCKED_TEXT_COLOR),
            ));
            return;
        }

        for (&level, stats) in &all_levels {
            let boss = boss_name_for_level(level);
            let label = if let Some(name) = boss {
                format!("Level {} ({})", level, name)
            } else {
                format!("Level {}", level)
            };
            let color = efficiency_color(stats.best_efficiency);

            // Level header
            parent.spawn((
                Text::new(&label),
                TextFont {
                    font_size: STAT_SECTION_FONT_SIZE,
                    ..default()
                },
                TextColor(color),
                Node {
                    margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(8.0), Val::Px(2.0)),
                    ..default()
                },
            ));

            spawn_stat_text_row(
                parent,
                "Efficiency",
                &format!("{:.0}%", stats.best_efficiency * 100.0),
            );
            spawn_stat_row(parent, "Attackers Killed", stats.attackers_killed);
            spawn_stat_row(parent, "Undead Killed", stats.undead_killed);
            spawn_stat_row(parent, "Defenders Lost", stats.defenders_lost);
            spawn_stat_text_row(parent, "Time", &format_time(stats.elapsed_time));
        }
    });
}

// ---------------------------------------------------------------------------
// Time Travel section
// ---------------------------------------------------------------------------

/// Initializes the time travel resource. Call before spawning the time travel UI.
pub(super) fn init_time_travel_resource(commands: &mut Commands) {
    commands.insert_resource(SelectedTimeTravelLevel::default());
}

fn spawn_time_travel_section(parent: &mut ChildSpawnerCommands, config: &GameConfig) {
    // Title
    parent.spawn((
        Text::new("Time Travel"),
        TextFont::from_font_size(20.0),
        TextColor(TIME_TRAVEL_BOSS_COLOR),
        Node {
            margin: UiRect::top(Val::Px(16.0)),
            ..default()
        },
    ));

    // Level list container with border
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                width: Val::Px(240.0),
                ..default()
            },
            BackgroundColor(TIME_TRAVEL_SECTION_BG),
            BorderColor::all(TIME_TRAVEL_SECTION_BORDER),
            TimeTravelContainer,
        ))
        .with_children(|section| {
            // Scrollable level list
            section
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        max_height: Val::Px(TIME_TRAVEL_LIST_MAX_HEIGHT),
                        width: Val::Percent(100.0),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    ScrollPosition::default(),
                    TimeTravelSection,
                ))
                .with_children(|list| {
                    for level in 1..config.highest_level_achieved {
                        let label = if let Some(boss) = boss_name_for_level(level) {
                            format!("Level {} ({})", level, boss)
                        } else {
                            format!("Level {}", level)
                        };
                        let text_color = if boss_name_for_level(level).is_some() {
                            TIME_TRAVEL_BOSS_COLOR
                        } else {
                            TEXT_COLOR
                        };

                        list.spawn((
                            Button,
                            Node {
                                height: Val::Px(TIME_TRAVEL_LEVEL_HEIGHT),
                                padding: UiRect::horizontal(Val::Px(8.0)),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            TimeTravelLevelButton(level),
                        ))
                        .with_child((
                            Text::new(label),
                            TextFont::from_font_size(TIME_TRAVEL_LEVEL_FONT_SIZE),
                            TextColor(text_color),
                        ));
                    }
                });

            // Selected level display
            section.spawn((
                Text::new("Select a level..."),
                TextFont::from_font_size(TIME_TRAVEL_LEVEL_FONT_SIZE),
                TextColor(TEXT_COLOR),
                TimeTravelSelectedDisplay,
            ));

            // Start button
            spawn_button(
                section,
                "Start Time Travel",
                EndlessAction::StartTimeTravel,
                &START_TIME_TRAVEL_BUTTON_STYLE,
            );
        });
}

// ---------------------------------------------------------------------------
// Time travel interaction systems
// ---------------------------------------------------------------------------

/// Handles clicks on time travel level buttons.
pub(super) fn handle_time_travel_level_clicks(
    mut button_clicked: MessageReader<MouseClicked>,
    level_buttons: Query<&TimeTravelLevelButton>,
    mut selected: ResMut<SelectedTimeTravelLevel>,
    mut level_button_nodes: Query<(&TimeTravelLevelButton, &mut BackgroundColor, &Children)>,
    grandchildren_query: Query<&Children>,
    mut text_queries: ParamSet<(
        Query<(&mut Text, &mut TextColor), With<TimeTravelSelectedDisplay>>,
        Query<&mut TextColor>,
    )>,
) {
    for event in button_clicked.read() {
        if let Ok(btn) = level_buttons.get(event.button) {
            let selected_level = btn.0;
            selected.0 = Some(selected_level);

            // Update display text
            for (mut text, mut color) in &mut text_queries.p0() {
                text.0 = format!("Selected: Level {}", selected_level);
                color.0 = TIME_TRAVEL_SELECTED_TEXT;
            }

            // Collect descendant updates (children + grandchildren for shadow wrappers)
            let mut updates: Vec<(Entity, Color)> = Vec::new();
            for (lb, _, children) in &level_button_nodes {
                let is_selected = lb.0 == selected_level;
                let color = if is_selected {
                    TIME_TRAVEL_SELECTED_TEXT
                } else if boss_name_for_level(lb.0).is_some() {
                    TIME_TRAVEL_BOSS_COLOR
                } else {
                    TEXT_COLOR
                };
                for child in children.iter() {
                    updates.push((child, color));
                    if let Ok(gcs) = grandchildren_query.get(child) {
                        for gc in gcs.iter() {
                            updates.push((gc, color));
                        }
                    }
                }
            }

            // Apply background highlights
            for (lb, mut bg, _) in &mut level_button_nodes {
                bg.0 = if lb.0 == selected_level {
                    TIME_TRAVEL_SELECTED_BG
                } else {
                    Color::NONE
                };
            }

            // Apply text color updates
            let mut text_colors = text_queries.p1();
            for (entity, color) in updates {
                if let Ok(mut tc) = text_colors.get_mut(entity) {
                    tc.0 = color;
                }
            }
        }
    }
}

/// Handles hover effects on time travel level buttons.
pub(super) fn handle_time_travel_level_hover(
    mut level_buttons: Query<
        (&Interaction, &TimeTravelLevelButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    selected: Option<Res<SelectedTimeTravelLevel>>,
) {
    let selected_level = selected.and_then(|s| s.0);
    for (interaction, btn, mut bg) in &mut level_buttons {
        let is_selected = selected_level == Some(btn.0);
        bg.0 = match *interaction {
            Interaction::Hovered | Interaction::Pressed if !is_selected => TIME_TRAVEL_HOVER_BG,
            _ if is_selected => TIME_TRAVEL_SELECTED_BG,
            _ => Color::NONE,
        };
    }
}

// ---------------------------------------------------------------------------
// Endless action handler
// ---------------------------------------------------------------------------

/// Handles endless tab button actions.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_endless_actions(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    action_query: Query<&EndlessAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut kill_stats: ResMut<KillStats>,
    mut current_level: ResMut<CurrentLevel>,
    mut config: ResMut<GameConfig>,
    active_save: Res<crate::config::ActiveSave>,
    selected_tt_level: Option<Res<SelectedTimeTravelLevel>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
    mut right_panel_view: ResMut<RightPanelView>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = action_query.get(event.button) {
            match action {
                EndlessAction::ContinuePlay => {
                    commands.insert_resource(GameMode::Endless);
                    channel_change.write(ChannelChangeMessage);
                    kill_stats.reset();
                    next_app_state.set(AppState::Loading);
                }
                EndlessAction::SwitchWizardType => {
                    *right_panel_view = RightPanelView::WizardSelect;
                }
                EndlessAction::StartTimeTravel => {
                    if let Some(ref sel) = selected_tt_level
                        && let Some(level) = sel.0
                    {
                        // Store the real level so it can be restored after time travel.
                        // Don't modify config.current_level — that tracks actual progression.
                        commands.insert_resource(TimeTravelState {
                            real_level: config.current_level,
                        });
                        commands.insert_resource(GameMode::Endless);
                        // Load the terrain snapshot for the target level so the
                        // battlefield looks like it did when that level was first played.
                        crate::config::save_data::load_level_terrain_into_config(
                            &active_save,
                            level,
                            &mut config,
                        );
                        current_level.0 = level;
                        channel_change.write(ChannelChangeMessage);
                        kill_stats.reset();
                        next_app_state.set(AppState::Loading);
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Aggregates best endless stats for a wizard type across all matching wizard saves.
fn aggregate_endless_stats(
    save: &Option<crate::config::save_data::UnifiedSaveFile>,
    wizard_type: WizardType,
) -> std::collections::BTreeMap<u32, EndlessLevelBest> {
    let mut all_levels: std::collections::BTreeMap<u32, EndlessLevelBest> =
        std::collections::BTreeMap::new();

    if let Some(save) = save {
        for wizard in &save.wizards {
            if wizard.wizard_type != wizard_type {
                continue;
            }
            for (level_str, stats) in &wizard.endless_best_stats {
                if let Ok(level) = level_str.parse::<u32>() {
                    all_levels
                        .entry(level)
                        .and_modify(|existing| {
                            if stats.best_efficiency > existing.best_efficiency {
                                *existing = stats.clone();
                            }
                        })
                        .or_insert_with(|| stats.clone());
                }
            }
        }
    }

    all_levels
}

/// Returns a color for the efficiency value:
/// - 100%: gold glow
/// - 0-99%: gradient from red (0%) to green (99%)
use crate::ui::constants::efficiency_color;

/// Spawns a stat row with a label and numeric value.
fn spawn_stat_row(parent: &mut ChildSpawnerCommands, label: &str, value: u32) {
    spawn_stat_value_row(parent, label, &format!("{}", value), TEXT_COLOR);
}

/// Spawns a stat row with a label and text value.
fn spawn_stat_text_row(parent: &mut ChildSpawnerCommands, label: &str, value: &str) {
    spawn_stat_value_row(parent, label, value, TEXT_COLOR);
}

/// Spawns a row with a label on the left and a value on the right.
fn spawn_stat_value_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: &str,
    value_color: Color,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            width: Val::Percent(100.0),
            padding: UiRect::horizontal(Val::Px(4.0)),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont::from_font_size(STAT_VALUE_FONT_SIZE),
                TextColor(STAT_LABEL_COLOR),
            ));
            row.spawn((
                Text::new(value),
                TextFont::from_font_size(STAT_VALUE_FONT_SIZE),
                TextColor(value_color),
                Node {
                    min_width: Val::Px(40.0),
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
                },
                TextLayout::new_with_justify(Justify::Right),
            ));
        });
}
