use bevy::prelude::*;

use crate::config::save_data::{EndlessLevelBest, load_unified_save};
use crate::config::{GameConfig, WizardType};
use crate::game::constants::boss_name_for_level;
use crate::game::game_mode::components::format_time;
use crate::ui::constants::efficiency_color;
use crate::ui::systems::spawn_button;

use super::super::components::*;
use super::super::constants::*;
use super::actions::EndlessAction;
use super::time_travel::init_time_travel_resource;
use super::time_travel::spawn_time_travel_section;

// ---------------------------------------------------------------------------
// Right panel builder
// ---------------------------------------------------------------------------

/// Populates the right panel with endless mode content:
/// - "Switch Wizard Type" button
/// - "Continue Playing" button
/// - Time Travel section (if player has beaten at least level 1)
pub(crate) fn build_endless_right_panel(
    commands: &mut Commands,
    right_panel_entity: Entity,
    config: &GameConfig,
    // Co-op gating: None = no guest (solo); Some(false) = guest not ready;
    // Some(true) = guest ready. When Some(false) the start button is disabled.
    guest_pending: Option<bool>,
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
                        let continue_label = if config.highest_level_achieved <= 1 {
                            "Start"
                        } else {
                            "Continue Playing"
                        };
                        spawn_coop_gated_button(
                            left_col,
                            guest_pending,
                            continue_label,
                            EndlessAction::ContinuePlay,
                            &BUTTON_STYLE,
                        );

                        spawn_button(
                            left_col,
                            "Switch Wizard",
                            EndlessAction::SwitchWizardType,
                            &SWITCH_WIZARD_BUTTON_STYLE,
                        );

                        #[cfg(debug_assertions)]
                        {
                            use super::actions::DebugLevelButtons;
                            left_col
                                .spawn((
                                    Node {
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(4.0),
                                        ..default()
                                    },
                                    Visibility::Hidden,
                                    DebugLevelButtons,
                                ))
                                .with_children(|wrapper| {
                                    spawn_button(
                                        wrapper,
                                        "Debug: Level +1",
                                        EndlessAction::DebugIncreaseLevel,
                                        &DEBUG_BUTTON_STYLE,
                                    );
                                    spawn_button(
                                        wrapper,
                                        "Debug: Level -1",
                                        EndlessAction::DebugDecreaseLevel,
                                        &DEBUG_BUTTON_STYLE,
                                    );
                                });
                        }
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
pub(crate) fn build_endless_left_panel(
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
