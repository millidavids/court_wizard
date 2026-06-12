use bevy::prelude::*;

use crate::config::WizardType;
use crate::ui::constants::efficiency_color;

use super::super::components::*;
use super::super::constants::*;
use super::super::rows::{spawn_item_button, spawn_stat_row, spawn_stat_text_row};

pub(super) fn spawn_endless_items(
    parent: &mut ChildSpawnerCommands,
    save: Option<&crate::config::save_data::UnifiedSaveFile>,
    state: &CompendiumState,
) {
    // Show wizard types that have endless data as clickable items
    let mut wizard_types_with_data: Vec<(WizardType, u32)> = Vec::new();

    if let Some(save) = save {
        // Collect wizard types that have endless stats, with their highest level
        let mut type_levels: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();
        for wizard in &save.wizards {
            if wizard.endless_best_stats.is_empty() {
                continue;
            }
            let type_name = wizard.wizard_type.save_key().to_string();
            let max_level = wizard
                .endless_best_stats
                .keys()
                .filter_map(|k| k.parse::<u32>().ok())
                .max()
                .unwrap_or(0);
            let entry = type_levels.entry(type_name).or_insert(0);
            if max_level > *entry {
                *entry = max_level;
            }
        }

        for wizard_type in WizardType::all() {
            let debug_name = wizard_type.save_key().to_string();
            if let Some(&highest) = type_levels.get(&debug_name) {
                wizard_types_with_data.push((*wizard_type, highest));
            }
        }
    }

    if wizard_types_with_data.is_empty() {
        parent.spawn((
            Text::new("No endless levels completed yet.\nPlay Endless mode to track your best stats per level."),
            TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
            TextColor(LOCKED_COLOR),
        ));
        return;
    }

    for (wizard_type, highest_level) in &wizard_types_with_data {
        let debug_name = wizard_type.save_key().to_string();
        let label = format!("{} (Lv{})", wizard_type.display_name(), highest_level);
        spawn_item_button(
            parent,
            &label,
            UNLOCKED_COLOR,
            CompendiumItemId::EndlessWizardType(debug_name),
            &state.selected_item,
        );
    }
}

pub(super) fn spawn_endless_detail_for_wizard(
    parent: &mut ChildSpawnerCommands,
    save: Option<&crate::config::save_data::UnifiedSaveFile>,
    wizard_type_name: &str,
) {
    use crate::game::game_mode::components::format_time;

    // Aggregate best stats for this wizard type across all wizard saves of that type
    let mut all_levels: std::collections::BTreeMap<
        u32,
        crate::config::save_data::EndlessLevelBest,
    > = std::collections::BTreeMap::new();

    if let Some(save) = save {
        for wizard in &save.wizards {
            if wizard.wizard_type.save_key() != wizard_type_name {
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

    if all_levels.is_empty() {
        parent.spawn((
            Text::new("No data yet."),
            TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
            TextColor(LOCKED_COLOR),
        ));
        return;
    }

    for (&level, stats) in &all_levels {
        let boss = crate::game::constants::boss_name_for_level(level);
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
}
