use bevy::prelude::*;

use crate::ui::constants::efficiency_color;

use super::super::components::*;
use super::super::constants::*;
use super::super::rows::{
    spawn_item_button, spawn_stat_row, spawn_stat_section_header, spawn_stat_text_row,
};

/// Collects all roguelite runs across all wizards, sorted by most recent first.
pub(super) fn collect_all_roguelite_runs(
    save: Option<&crate::config::save_data::UnifiedSaveFile>,
) -> Vec<&crate::config::save_data::RogueliteRun> {
    let mut all_runs: Vec<&crate::config::save_data::RogueliteRun> = Vec::new();
    if let Some(save) = save {
        for wizard in &save.wizards {
            all_runs.extend(wizard.roguelite.run_history.iter());
        }
    }
    all_runs.sort_by_key(|run| std::cmp::Reverse(run.ended_at));
    all_runs
}

pub(super) fn spawn_roguelite_items(
    parent: &mut ChildSpawnerCommands,
    save: Option<&crate::config::save_data::UnifiedSaveFile>,
    state: &CompendiumState,
) {
    let all_runs = collect_all_roguelite_runs(save);

    if all_runs.is_empty() {
        parent.spawn((
            Text::new("No roguelite runs completed yet.\nPlay Roguelite mode to track your runs."),
            TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
            TextColor(LOCKED_COLOR),
        ));
        return;
    }

    let total_runs = all_runs.len();
    let victories = all_runs.iter().filter(|r| r.victory).count();

    // Summary header
    parent.spawn((
        Text::new(format!("{} runs — {} victories", total_runs, victories)),
        TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
        TextColor(DESCRIPTION_COLOR),
        Node {
            margin: UiRect::bottom(Val::Px(4.0)),
            ..default()
        },
    ));

    // Saved runs section
    let saved_runs: Vec<(usize, &crate::config::save_data::RogueliteRun)> = all_runs
        .iter()
        .enumerate()
        .filter(|(_, r)| r.saved)
        .map(|(i, r)| (total_runs - i, *r))
        .collect();

    if !saved_runs.is_empty() {
        spawn_stat_section_header(parent, "Saved Runs");
        for (run_number, run) in &saved_runs {
            spawn_roguelite_run_button(parent, run, state, *run_number);
        }
    }

    // Recent (unsaved) runs section
    let recent_runs: Vec<(usize, &crate::config::save_data::RogueliteRun)> = all_runs
        .iter()
        .enumerate()
        .filter(|(_, r)| !r.saved)
        .take(20)
        .map(|(i, r)| (total_runs - i, *r))
        .collect();

    if !recent_runs.is_empty() {
        spawn_stat_section_header(parent, "Recent Runs");
        for (run_number, run) in &recent_runs {
            spawn_roguelite_run_button(parent, run, state, *run_number);
        }
    }
}

pub(super) fn spawn_roguelite_run_button(
    parent: &mut ChildSpawnerCommands,
    run: &crate::config::save_data::RogueliteRun,
    state: &CompendiumState,
    run_number: usize,
) {
    let outcome = if run.victory { "Victory" } else { "Defeat" };
    let outcome_color = if run.victory {
        Color::srgb(0.3, 0.8, 0.3)
    } else {
        Color::srgb(0.8, 0.3, 0.3)
    };

    let assist_marker = if run.accessibility_assists { " *" } else { "" };
    let label = format!(
        "Run #{} — {} (Lv{}, {}){}",
        run_number,
        outcome,
        run.levels_completed,
        run.wizard_type.display_name(),
        assist_marker,
    );
    spawn_item_button(
        parent,
        &label,
        outcome_color,
        CompendiumItemId::RogueliteRun(run.started_at),
        &state.selected_item,
    );
}

pub(super) fn spawn_roguelite_run_detail(
    parent: &mut ChildSpawnerCommands,
    save: Option<&crate::config::save_data::UnifiedSaveFile>,
    started_at: u64,
) {
    use crate::game::game_mode::components::{RunAggregateStats, format_time};

    // Find the run by started_at timestamp directly instead of re-sorting all runs
    let run = save.and_then(|s| {
        s.wizards
            .iter()
            .flat_map(|w| w.roguelite.run_history.iter())
            .find(|r| r.started_at == started_at)
    });

    let Some(run) = run else {
        parent.spawn((
            Text::new("Run not found."),
            TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
            TextColor(LOCKED_COLOR),
        ));
        return;
    };

    // Save/Unsave button
    let (btn_label, btn_color) = if run.saved {
        ("Unsave Run", Color::srgb(0.8, 0.3, 0.3))
    } else {
        ("Save Run", Color::srgb(0.3, 0.6, 0.9))
    };
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(6.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                margin: UiRect::bottom(Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(ITEM_BG),
            BorderColor::all(btn_color),
            crate::ui::components::ButtonColors {
                background: ITEM_BG,
                border: btn_color,
            },
            ToggleSaveRunButton(run.started_at),
            crate::ui::focus::Focusable,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(btn_label),
                TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                TextColor(btn_color),
            ));
        });

    // Run summary stats
    let agg = RunAggregateStats::from_level_stats(&run.level_stats);
    spawn_stat_section_header(parent, "Run Summary");
    spawn_stat_row(parent, "Levels Completed", run.levels_completed);
    spawn_stat_row(parent, "Total Kills", agg.total_kills);
    spawn_stat_text_row(
        parent,
        "Avg Efficiency",
        &format!("{:.0}%", agg.avg_efficiency * 100.0),
    );
    spawn_stat_text_row(parent, "Total Time", &format_time(agg.total_time));

    // Seed with copy button
    if let Some(seed) = run.seed {
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|col| {
                col.spawn((
                    Text::new("Seed"),
                    TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
                    TextColor(STAT_LABEL_COLOR),
                ));
                col.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new(seed.to_string()),
                        TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
                        TextColor(TEXT_COLOR),
                    ));
                    row.spawn((
                        Button,
                        Node {
                            padding: UiRect::new(
                                Val::Px(6.0),
                                Val::Px(6.0),
                                Val::Px(2.0),
                                Val::Px(2.0),
                            ),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                            ..default()
                        },
                        BackgroundColor(ITEM_BG),
                        BorderColor::all(Color::srgb(0.3, 0.6, 0.9)),
                        crate::ui::components::ButtonColors {
                            background: ITEM_BG,
                            border: Color::srgb(0.3, 0.6, 0.9),
                        },
                        CopySeedButton(seed),
                        crate::ui::focus::Focusable,
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Copy"),
                            TextFont::from_font_size(10.0),
                            TextColor(Color::srgb(0.3, 0.6, 0.9)),
                        ));
                    });
                });
            });
    }

    // Modifier display
    if let Some(ref mods) = run.modifiers {
        spawn_stat_section_header(parent, "Run Settings");
        spawn_stat_text_row(
            parent,
            "Wave Speed",
            &format!("{}%", (mods.game_speed * 100.0) as u32),
        );
        spawn_stat_text_row(
            parent,
            "Enemy Strength",
            &format!("{}%", (mods.enemy_effectiveness * 100.0) as u32),
        );
        spawn_stat_text_row(
            parent,
            "Enemy Count",
            &format!("{}%", (mods.enemy_count * 100.0) as u32),
        );
        spawn_stat_text_row(
            parent,
            "Terrain",
            &format!("{}%", (mods.terrain_density * 100.0) as u32),
        );
    }

    if run.accessibility_assists {
        spawn_stat_text_row(parent, "Accessibility", "Assists Used");
    }

    // Level-by-level breakdown
    spawn_stat_section_header(parent, "Level Breakdown");
    for stats in &run.level_stats {
        let color = efficiency_color(stats.efficiency);
        let label = format!("Level {}:", stats.level);
        let value = format!(
            "{:.0}% — {} kills, {}",
            stats.efficiency * 100.0,
            stats.total_kills(),
            format_time(stats.elapsed_time),
        );

        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(4.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new(label),
                    TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
                    TextColor(color),
                    Node {
                        flex_shrink: 0.0,
                        min_width: Val::Px(70.0),
                        ..default()
                    },
                ));
                row.spawn((
                    Text::new(value),
                    TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
                    TextColor(color),
                    Node {
                        flex_shrink: 1.0,
                        ..default()
                    },
                ));
            });
    }
}
