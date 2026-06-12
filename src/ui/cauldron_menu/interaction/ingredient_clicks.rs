use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::config::save_data::load_unified_save;
use crate::config::{GameConfig, WizardType};
use crate::game::cauldron::brews::Recipe;
use crate::game::cauldron::brews::constants::{
    ALCHEMIST_BREW_TIME_MULTIPLIER, ALCHEMIST_DURATION_MULTIPLIER,
};
use crate::game::cauldron::components::{Cauldron, CauldronState};
use crate::ui::components::{ButtonActive, ButtonColors};
use crate::ui::systems::spawn_button;

/// Rebuilds just the detail panel content when the ingredient selection changes.
/// Avoids re-rendering the entire cauldron menu.
pub(crate) fn update_detail_panel_on_selection_change(
    mut commands: Commands,
    selection: Res<IngredientSelection>,
    config: Res<GameConfig>,
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
    detail_panel: Query<Entity, With<CauldronDetailPanel>>,
    children_query: Query<&Children>,
    stone_query: Query<(), With<StoneSelectorPanel>>,
) {
    let Ok(panel_entity) = detail_panel.single() else {
        return;
    };
    let is_brewing = cauldron_query
        .single()
        .is_ok_and(|state| state.is_brewing());
    if is_brewing {
        return; // Don't update detail panel during brewing
    }

    // Rebuild the preview content, but PRESERVE the persistent Stone selector
    // (the panel child marked `StoneSelectorPanel`). Despawning it would drop the
    // gamepad's focused entity when toggling the Stone and bounce focus into the
    // grid, so despawn every child EXCEPT the Stone selector.
    if let Ok(children) = children_query.get(panel_entity) {
        for child in children.iter() {
            if !stone_query.contains(child) {
                commands.entity(child).try_despawn();
            }
        }
    }

    let unlocked_combos = load_unified_save()
        .map(|s| s.player.unlocked_content.combos)
        .unwrap_or_default();

    commands.entity(panel_entity).with_children(|left| {
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
            if selection.is_empty() {
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
                let recipe = Recipe::new(selection.build_ingredients());
                let is_alchemist = config.wizard_type == WizardType::Alchemist;

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

                panel.spawn((
                    Text::new("Effects"),
                    TextFont::from_font_size(DETAIL_LABEL_FONT_SIZE),
                    TextColor(DETAIL_LABEL_COLOR),
                ));

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
                        panel.spawn((
                            Text::new(combo.name),
                            TextFont::from_font_size(COMBO_FONT_SIZE),
                            TextColor(COMBO_COLOR),
                        ));
                        panel.spawn((
                            Text::new(combo.description),
                            TextFont::from_font_size(EFFECT_PREVIEW_FONT_SIZE),
                            TextColor(DISABLED_TEXT_COLOR),
                        ));
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

        // Brew button (only when ingredients are selected)
        if !selection.is_empty() {
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

/// Keeps every ingredient + Philosopher's Stone toggle button's look in sync with
/// the current selection. Selected buttons show the active style; once the
/// 3-ingredient limit is reached, every unselected ingredient switches to the
/// disabled style (and back when one is removed). Runs in place — no despawn — so
/// a gamepad's focused button is never lost. This is the single source of truth
/// for toggle-button appearance, so `button_action` only mutates the selection.
pub(crate) fn sync_toggle_button_states(
    selection: Res<IngredientSelection>,
    mut commands: Commands,
    mut buttons: Query<(
        Entity,
        &CauldronMenuButtonAction,
        &mut ButtonColors,
        Has<ButtonActive>,
    )>,
) {
    let at_limit = selection.at_limit();
    for (entity, action, mut colors, has_active) in &mut buttons {
        let (active, style) = match action {
            CauldronMenuButtonAction::ToggleIngredient(ingredient) => {
                let is_selected = selection.is_selected(ingredient);
                let style = if is_selected {
                    &INGREDIENT_SELECTED_STYLE
                } else if at_limit {
                    &INGREDIENT_DISABLED_STYLE
                } else {
                    &INGREDIENT_BUTTON_STYLE
                };
                (is_selected, style)
            }
            CauldronMenuButtonAction::TogglePhilosophersStone => {
                let has_stone = selection.has_stone();
                let style = if has_stone {
                    &STONE_SELECTED_STYLE
                } else {
                    &STONE_BUTTON_STYLE
                };
                (has_stone, style)
            }
            _ => continue,
        };

        // Only write when the color actually differs, to avoid spurious
        // `Changed<ButtonColors>` churn through the button render systems.
        if colors.background != style.background || colors.border != style.border {
            colors.background = style.background;
            colors.border = style.border;
        }
        if active && !has_active {
            commands.entity(entity).insert(ButtonActive);
        } else if !active && has_active {
            commands.entity(entity).remove::<ButtonActive>();
        }
    }
}
