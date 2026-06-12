use bevy::prelude::*;

use crate::config::WizardType;
use crate::config::save_data::AchievementId;
use crate::game::cauldron::brews::Ingredient;
use crate::game::units::UnitType;
use crate::game::units::wizard::components::{Spell, SpellCategory};

use super::super::components::*;
use super::super::constants::*;
use super::super::rows::{
    spawn_insight_row, spawn_item_button, spawn_stat_row, spawn_stat_section_header,
    spawn_stat_text_row,
};

pub(crate) fn team_label_color(label: &str) -> Color {
    match label {
        "Defender" => TEAM_DEFENDER_COLOR,
        "Attacker" => TEAM_ATTACKER_COLOR,
        "Boss" => TEAM_BOSS_COLOR,
        _ => DESCRIPTION_COLOR,
    }
}

pub(super) fn spawn_spell_items(
    parent: &mut ChildSpawnerCommands,
    unlocked_spells: &[String],
    research_progress: &std::collections::HashMap<String, u32>,
    state: &CompendiumState,
) {
    for category in SpellCategory::all() {
        // Category header
        parent.spawn((
            Text::new(category.display_name()),
            TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
            TextColor(crate::ui::constants::spell_category_color(*category)),
            Node {
                margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(6.0), Val::Px(2.0)),
                ..default()
            },
        ));

        for spell in category.spells() {
            let debug_name = spell.save_key().to_string();
            let is_unlocked = spell.research_cost() == 0 || unlocked_spells.contains(&debug_name);
            let progress = research_progress.get(&debug_name).copied().unwrap_or(0);
            let cost = spell.research_cost();

            let display_text = if is_unlocked {
                spell.display_name().to_string()
            } else if progress > 0 {
                format!("{} ({}/{})", spell.display_name(), progress, cost)
            } else {
                "???".to_string()
            };

            let text_color = if is_unlocked {
                UNLOCKED_COLOR
            } else if progress > 0 {
                IN_PROGRESS_COLOR
            } else {
                LOCKED_COLOR
            };

            spawn_item_button(
                parent,
                &display_text,
                text_color,
                CompendiumItemId::Spell(debug_name),
                &state.selected_item,
            );
        }
    }
}

pub(super) fn spawn_ingredient_items(
    parent: &mut ChildSpawnerCommands,
    unlocked_ingredients: &[String],
    state: &CompendiumState,
) {
    let mut ingredients: Vec<_> = Ingredient::all()
        .iter()
        .map(|i| {
            let debug_name = i.save_key().to_string();
            let is_unlocked = unlocked_ingredients.contains(&debug_name);
            (i, debug_name, is_unlocked)
        })
        .collect();
    ingredients.sort_by(|(_, _, a), (_, _, b)| b.cmp(a));

    for (ingredient, debug_name, is_unlocked) in ingredients {
        let display_text = if is_unlocked {
            ingredient.name().to_string()
        } else {
            "???".to_string()
        };
        let text_color = if is_unlocked {
            UNLOCKED_COLOR
        } else {
            LOCKED_COLOR
        };
        spawn_item_button(
            parent,
            &display_text,
            text_color,
            CompendiumItemId::Ingredient(debug_name),
            &state.selected_item,
        );
    }
}

pub(super) fn spawn_unit_items(
    parent: &mut ChildSpawnerCommands,
    unlocked_units: &[String],
    state: &CompendiumState,
) {
    // Group by team label
    for team_label in &["Defender", "Attacker", "Boss"] {
        parent.spawn((
            Text::new(*team_label),
            TextFont::from_font_size(ITEM_NAME_FONT_SIZE),
            TextColor(team_label_color(team_label)),
            Node {
                margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(6.0), Val::Px(2.0)),
                ..default()
            },
        ));

        for unit_type in UnitType::all() {
            if unit_type.team_label() != *team_label {
                continue;
            }
            let debug_name = unit_type.save_key().to_string();
            let is_unlocked = unlocked_units.contains(&debug_name);

            let display_text = if is_unlocked {
                unit_type.display_name().to_string()
            } else {
                "???".to_string()
            };
            let text_color = if is_unlocked {
                UNLOCKED_COLOR
            } else {
                LOCKED_COLOR
            };
            spawn_item_button(
                parent,
                &display_text,
                text_color,
                CompendiumItemId::Unit(debug_name),
                &state.selected_item,
            );
        }
    }
}

pub(super) fn spawn_wizard_items(
    parent: &mut ChildSpawnerCommands,
    unlocked_wizard_types: &[String],
    state: &CompendiumState,
) {
    for wizard_type in WizardType::all() {
        let debug_name = wizard_type.save_key().to_string();
        let is_unlocked = *wizard_type == WizardType::BoringOleMage
            || unlocked_wizard_types.contains(&debug_name);

        let display_text = if is_unlocked {
            wizard_type.display_name().to_string()
        } else {
            "???".to_string()
        };
        let text_color = if is_unlocked {
            UNLOCKED_COLOR
        } else {
            LOCKED_COLOR
        };
        spawn_item_button(
            parent,
            &display_text,
            text_color,
            CompendiumItemId::Wizard(debug_name),
            &state.selected_item,
        );
    }
}

pub(super) fn spawn_achievement_items(
    parent: &mut ChildSpawnerCommands,
    unlocked_achievements: &[String],
    state: &CompendiumState,
) {
    let mut achievements: Vec<_> = AchievementId::all()
        .iter()
        .map(|a| {
            let is_unlocked = unlocked_achievements.contains(&a.id().to_string());
            (a, is_unlocked)
        })
        .collect();
    achievements.sort_by(|(a, a_unlocked), (b, b_unlocked)| {
        b_unlocked
            .cmp(a_unlocked)
            .then_with(|| a.display_name().cmp(b.display_name()))
    });

    for (achievement, is_unlocked) in achievements {
        let display_text = if is_unlocked {
            achievement.display_name().to_string()
        } else {
            "???".to_string()
        };
        let text_color = if is_unlocked {
            UNLOCKED_COLOR
        } else {
            LOCKED_COLOR
        };
        spawn_item_button(
            parent,
            &display_text,
            text_color,
            CompendiumItemId::Achievement(achievement.id().to_string()),
            &state.selected_item,
        );
    }
}

pub(super) fn spawn_stats_items(
    parent: &mut ChildSpawnerCommands,
    save: Option<&crate::config::save_data::UnifiedSaveFile>,
) {
    let total_games = save.map(|s| s.player.total_games_played).unwrap_or(0);
    let total_victories = save.map(|s| s.player.total_levels_completed).unwrap_or(0);
    let total_attackers = save.map(|s| s.player.total_attackers_killed).unwrap_or(0);
    let total_defenders = save.map(|s| s.player.total_defenders_killed).unwrap_or(0);
    let total_undead = save.map(|s| s.player.total_undead_killed).unwrap_or(0);
    let insight_balance = save.map(|s| s.player.arcane_insight).unwrap_or(0);

    // Highest level achieved across all wizards
    let highest_level = save
        .map(|s| {
            s.wizards
                .iter()
                .map(|w| w.highest_level_achieved)
                .max()
                .unwrap_or(1)
        })
        .unwrap_or(1);

    // Total wizards created
    let wizards_created = save.map(|s| s.wizards.len() as u32).unwrap_or(0);

    // Spells unlocked
    let spells_unlocked = save
        .map(|s| s.player.unlocked_content.spells.len() as u32)
        .unwrap_or(0);
    let total_spells = Spell::all().len() as u32;

    // Ingredients unlocked
    let ingredients_unlocked = save
        .map(|s| s.player.unlocked_content.ingredients.len() as u32)
        .unwrap_or(0);
    let total_ingredients = Ingredient::all().len() as u32;

    // Achievements unlocked
    let achievements_unlocked = save
        .map(|s| s.player.unlocked_achievements.len() as u32)
        .unwrap_or(0);
    let total_achievements = AchievementId::all().len() as u32;

    // Talents unlocked (count non-negative selections across all spells)
    let talents_unlocked = save
        .map(|s| {
            s.player
                .spell_talent_selections
                .values()
                .flat_map(|v| v.iter())
                .filter(|&&sel| sel >= 0)
                .count() as u32
        })
        .unwrap_or(0);
    // Total possible talents = 3 tiers × number of spells that have talent trees
    // (all spells have talent trees)
    let total_talents = Spell::all().len() as u32 * 3;

    // Total kills
    let total_kills = total_attackers + total_undead;

    // Win rate
    let win_rate = if total_games > 0 {
        format!(
            "{:.0}%",
            (total_victories as f32 / total_games as f32) * 100.0
        )
    } else {
        "N/A".to_string()
    };

    // Two-column layout
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(COLUMN_GAP),
            ..default()
        })
        .with_children(|columns| {
            // Left column
            columns
                .spawn(Node {
                    width: Val::Percent(50.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|col| {
                    spawn_stat_section_header(col, "Battle Stats");
                    spawn_stat_row(col, "Games Played", total_games);
                    spawn_stat_row(col, "Victories", total_victories);
                    spawn_stat_text_row(col, "Win Rate", &win_rate);
                    spawn_stat_row(col, "Highest Level", highest_level);

                    spawn_stat_section_header(col, "Kill Stats");
                    spawn_stat_row(col, "Total Kills", total_kills);
                    spawn_stat_row(col, "Attackers Killed", total_attackers);
                    spawn_stat_row(col, "Undead Killed", total_undead);
                    spawn_stat_row(col, "Defenders Lost", total_defenders);
                });

            // Right column
            columns
                .spawn(Node {
                    width: Val::Percent(50.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|col| {
                    spawn_stat_section_header(col, "Collection");
                    spawn_stat_text_row(
                        col,
                        "Spells Unlocked",
                        &format!("{}/{}", spells_unlocked, total_spells),
                    );
                    spawn_stat_text_row(
                        col,
                        "Ingredients Found",
                        &format!("{}/{}", ingredients_unlocked, total_ingredients),
                    );
                    spawn_stat_text_row(
                        col,
                        "Talents Unlocked",
                        &format!("{}/{}", talents_unlocked, total_talents),
                    );
                    spawn_stat_text_row(
                        col,
                        "Achievements",
                        &format!("{}/{}", achievements_unlocked, total_achievements),
                    );
                    spawn_stat_row(col, "Wizards Created", wizards_created);

                    spawn_stat_section_header(col, "Economy");
                    spawn_insight_row(col, "Arcane Insight", insight_balance);
                });
        });
}
