use bevy::prelude::*;

use super::stone_selector::spawn_philosophers_stone_selector;
use crate::config::{GameConfig, WizardType};
use crate::game::cauldron::brews::Recipe;
use crate::game::cauldron::brews::constants::{
    ALCHEMIST_BREW_TIME_MULTIPLIER, ALCHEMIST_DURATION_MULTIPLIER,
};
use crate::game::cauldron::resources::PhilosophersStoneUsed;
use crate::ui::cauldron_menu::components::*;
use crate::ui::cauldron_menu::constants::*;
use crate::ui::systems::spawn_button;

/// Spawns the left detail panel showing brew preview or brewing status.
pub(super) fn spawn_detail_panel(
    parent: &mut ChildSpawnerCommands,
    is_brewing: bool,
    selection: &IngredientSelection,
    unlocked_combos: &[String],
    config: &GameConfig,
    stone_used: &PhilosophersStoneUsed,
) {
    parent
        .spawn((
            Node {
                width: Val::Px(LEFT_PANEL_WIDTH),
                flex_direction: FlexDirection::Column,
                align_self: AlignSelf::Center,
                row_gap: Val::Px(16.0),
                flex_grow: 0.0,
                flex_shrink: 0.0,
                ..default()
            },
            CauldronDetailPanel,
        ))
        .with_children(|left| {
            // Philosopher's Stone selector at the top (Alchemist only, once per battle)
            spawn_philosophers_stone_selector(
                left,
                selection.has_stone(),
                !is_brewing && config.wizard_type == WizardType::Alchemist && !stone_used.0,
            );

            // Detail panel with border
            left.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(DETAIL_PADDING)),
                    row_gap: Val::Px(10.0),
                    border: UiRect::all(Val::Px(DETAIL_BORDER_WIDTH)),
                    border_radius: BorderRadius::all(Val::Px(DETAIL_BORDER_RADIUS)),
                    ..default()
                },
                BackgroundColor(DETAIL_BG),
                BorderColor::all(DETAIL_BORDER),
            ))
            .with_children(|panel| {
                if is_brewing {
                    // Brewing in progress
                    panel.spawn((
                        Text::new("Brewing in progress..."),
                        TextFont::from_font_size(BREWING_STATUS_FONT_SIZE),
                        TextColor(BREWING_STATUS_COLOR),
                    ));
                } else if selection.is_empty() {
                    // No selection placeholder
                    panel.spawn((
                        Text::new("Select ingredients to preview brew"),
                        TextFont::from_font_size(EFFECT_PREVIEW_FONT_SIZE),
                        TextColor(PLACEHOLDER_TEXT_COLOR),
                        Node {
                            max_width: Val::Px(LEFT_PANEL_WIDTH - DETAIL_PADDING * 2.0),
                            ..default()
                        },
                    ));
                } else {
                    // Show brew preview
                    let recipe = Recipe::new(selection.build_ingredients());
                    let is_alchemist = config.wizard_type == WizardType::Alchemist;

                    // Ingredient count (don't count Stone toward limit)
                    let count_color = if selection.at_limit() {
                        INGREDIENT_COUNT_FULL_COLOR
                    } else {
                        INGREDIENT_COUNT_COLOR
                    };
                    let mut count_text = format!(
                        "{}/{} ingredients",
                        selection.selected.len(),
                        MAX_INGREDIENTS
                    );
                    if selection.has_stone() {
                        count_text.push_str(" + Stone");
                    }
                    panel.spawn((
                        Text::new(count_text),
                        TextFont::from_font_size(DETAIL_LABEL_FONT_SIZE),
                        TextColor(count_color),
                    ));

                    // Brew time + duration (with Alchemist bonuses)
                    let brew_time = if is_alchemist {
                        recipe.brew_time() * ALCHEMIST_BREW_TIME_MULTIPLIER
                    } else {
                        recipe.brew_time()
                    };
                    let brew_time_text = if is_alchemist {
                        format!("Brew time: {:.0}s (20% faster)", brew_time)
                    } else {
                        format!("Brew time: {:.0}s", brew_time)
                    };
                    panel.spawn((
                        Text::new(brew_time_text),
                        TextFont::from_font_size(BREW_INFO_FONT_SIZE),
                        TextColor(BREW_INFO_COLOR),
                    ));

                    let duration = if is_alchemist {
                        recipe.buff_duration() * ALCHEMIST_DURATION_MULTIPLIER
                    } else {
                        recipe.buff_duration()
                    };
                    let duration_text = if is_alchemist {
                        format!("Duration: {:.0}s (25% longer)", duration)
                    } else {
                        format!("Duration: {:.0}s", duration)
                    };
                    panel.spawn((
                        Text::new(duration_text),
                        TextFont::from_font_size(BREW_INFO_FONT_SIZE),
                        TextColor(BREW_INFO_COLOR),
                    ));

                    // Effects label
                    panel.spawn((
                        Text::new("Effects"),
                        TextFont::from_font_size(DETAIL_LABEL_FONT_SIZE),
                        TextColor(DETAIL_LABEL_COLOR),
                    ));

                    // Effect list (skip no-op effects like Stone's BuffDurationMultiplier(1.0))
                    for effect in &recipe.base_effects() {
                        if effect.is_noop() {
                            continue;
                        }
                        panel.spawn((
                            Text::new(effect.display_text()),
                            TextFont::from_font_size(EFFECT_PREVIEW_FONT_SIZE),
                            TextColor(EFFECT_PREVIEW_COLOR),
                        ));
                    }

                    // Dilution warning
                    if recipe.ingredients.len() > 1 {
                        let dilution = recipe.dilution_factor();
                        let dilution_text = if selection.has_stone() {
                            "No dilution (Philosopher's Stone)".to_string()
                        } else {
                            format!("Dilution: {:.0}% strength", dilution * 100.0)
                        };
                        let dilution_color = if selection.has_stone() {
                            STONE_SELECTED_STYLE.text_color
                        } else {
                            DISABLED_TEXT_COLOR
                        };
                        panel.spawn((
                            Text::new(dilution_text),
                            TextFont::from_font_size(EFFECT_PREVIEW_FONT_SIZE),
                            TextColor(dilution_color),
                        ));
                    }

                    // Combo bonuses — only show previously unlocked combos
                    let visible_combos: Vec<_> = recipe
                        .matching_combos()
                        .into_iter()
                        .filter(|c| unlocked_combos.contains(&c.name.to_string()))
                        .collect();
                    if !visible_combos.is_empty() {
                        panel.spawn((
                            Text::new("Combos"),
                            TextFont::from_font_size(DETAIL_LABEL_FONT_SIZE),
                            TextColor(DETAIL_LABEL_COLOR),
                        ));
                        for combo in &visible_combos {
                            // Combo name
                            panel.spawn((
                                Text::new(combo.name),
                                TextFont::from_font_size(COMBO_FONT_SIZE),
                                TextColor(COMBO_COLOR),
                            ));
                            // Combo description
                            panel.spawn((
                                Text::new(combo.description),
                                TextFont::from_font_size(EFFECT_PREVIEW_FONT_SIZE),
                                TextColor(DISABLED_TEXT_COLOR),
                            ));
                            // Combo bonus effects
                            for effect in combo.bonus_effects {
                                panel.spawn((
                                    Text::new(format!("  {}", effect.display_text())),
                                    TextFont::from_font_size(EFFECT_PREVIEW_FONT_SIZE),
                                    TextColor(EFFECT_PREVIEW_COLOR),
                                ));
                            }
                        }
                    }
                }
            });

            // Brew button below the detail panel (only when ingredients are selected)
            if !is_brewing && !selection.is_empty() {
                spawn_button(
                    left,
                    "Brew",
                    (
                        CauldronMenuButtonAction::StartBrew,
                        crate::ui::focus::CrossRowHorizontalNav,
                    ),
                    &BREW_BUTTON_STYLE,
                );
            }
        });
}
