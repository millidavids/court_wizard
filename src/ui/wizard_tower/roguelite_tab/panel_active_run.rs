use bevy::prelude::*;

use crate::config::GameConfig;
use crate::game::game_mode::components::{RogueliteRunState, RunAggregateStats, format_time};
use crate::ui::systems::spawn_button;

use super::super::constants::SECTION_PADDING;
use super::super::constants::spawn_coop_gated_button;
use super::components::RogueliteAction;
use super::constants::{
    CONTINUE_RUN_BUTTON_STYLE, END_RUN_BUTTON_STYLE, LABEL_COLOR, RUN_STATS_LABEL_FONT,
    RUN_STATS_VALUE_COLOR, RUN_STATS_VALUE_FONT, SUMMARY_TITLE_FONT_SIZE,
};

use super::super::constants::TEXT_COLOR;

/// Builds the right panel content for the "active run" state.
/// Shows Continue and End Run buttons.
pub(crate) fn build_roguelite_active_run_right_panel(
    commands: &mut Commands,
    right_panel_entity: Entity,
    // Co-op gating: Some(false) disables Continue Run ("Guest Not Ready").
    guest_pending: Option<bool>,
) {
    commands
        .entity(right_panel_entity)
        .with_children(|wrapper| {
            wrapper
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(SECTION_PADDING)),
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|right| {
                    // Centered button container
                    right
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(16.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            flex_grow: 1.0,
                            ..default()
                        })
                        .with_children(|buttons| {
                            spawn_coop_gated_button(
                                buttons,
                                guest_pending,
                                "Continue Run",
                                RogueliteAction::ContinueRun,
                                &CONTINUE_RUN_BUTTON_STYLE,
                            );
                            spawn_button(
                                buttons,
                                "Abandon Run",
                                RogueliteAction::EndRun,
                                &END_RUN_BUTTON_STYLE,
                            );
                        });
                });
        });
}

/// Builds the left panel content for the "active run" state.
/// Shows current wizard, level progress, per-level stats, and aggregates.
pub(crate) fn build_roguelite_active_run_left_panel(
    commands: &mut Commands,
    left_panel_entity: Entity,
    config: &GameConfig,
    run_state: &RogueliteRunState,
) {
    commands.entity(left_panel_entity).with_children(|panel| {
        // Wizard name
        panel.spawn((
            Text::new(config.wizard_type.display_name()),
            TextFont::from_font_size(SUMMARY_TITLE_FONT_SIZE),
            TextColor(LABEL_COLOR),
        ));

        // Current level
        let levels_done = run_state.level_stats.len() as u32;
        let next_level = levels_done + 1;
        panel.spawn((
            Text::new(format!("Level {} (next: {})", levels_done, next_level)),
            TextFont::from_font_size(RUN_STATS_LABEL_FONT),
            TextColor(TEXT_COLOR),
            Node {
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },
        ));

        // Level-by-level stats
        if !run_state.level_stats.is_empty() {
            panel.spawn((
                Text::new("Level Stats"),
                TextFont::from_font_size(RUN_STATS_LABEL_FONT),
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::top(Val::Px(12.0)),
                    ..default()
                },
            ));

            for stat in &run_state.level_stats {
                panel.spawn((
                    Text::new(format!(
                        "  Lv{}: {}% eff, {} kills, {}",
                        stat.level,
                        (stat.efficiency * 100.0) as u32,
                        stat.total_kills(),
                        format_time(stat.elapsed_time),
                    )),
                    TextFont::from_font_size(RUN_STATS_VALUE_FONT),
                    TextColor(RUN_STATS_VALUE_COLOR),
                ));
            }

            // Aggregate stats
            let agg = RunAggregateStats::from_level_stats(&run_state.level_stats);
            panel.spawn((
                Text::new("Totals"),
                TextFont::from_font_size(RUN_STATS_LABEL_FONT),
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::top(Val::Px(12.0)),
                    ..default()
                },
            ));
            panel.spawn((
                Text::new(format!("  Total Kills: {}", agg.total_kills)),
                TextFont::from_font_size(RUN_STATS_VALUE_FONT),
                TextColor(RUN_STATS_VALUE_COLOR),
            ));
            panel.spawn((
                Text::new(format!(
                    "  Avg Efficiency: {}%",
                    (agg.avg_efficiency * 100.0) as u32
                )),
                TextFont::from_font_size(RUN_STATS_VALUE_FONT),
                TextColor(RUN_STATS_VALUE_COLOR),
            ));
            panel.spawn((
                Text::new(format!("  Total Time: {}", format_time(agg.total_time))),
                TextFont::from_font_size(RUN_STATS_VALUE_FONT),
                TextColor(RUN_STATS_VALUE_COLOR),
            ));
        }
    });
}
