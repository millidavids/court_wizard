//! Cauldron menu UI setup.

use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::config::save_data::load_unified_save;
use crate::config::{GameConfig, WizardType};
use crate::game::cauldron::brews::constants::{
    ALCHEMIST_BREW_TIME_MULTIPLIER, ALCHEMIST_DURATION_MULTIPLIER,
};
use crate::game::cauldron::brews::{Ingredient, IngredientCategory, Recipe};
use crate::game::cauldron::components::{Cauldron, CauldronState};
use crate::game::cauldron::resources::PhilosophersStoneUsed;
use crate::ui::systems::{spawn_button, spawn_page_container, spawn_title_with_shadow};

/// Spawns the cauldron menu UI when entering the CauldronMenu state.
pub(super) fn spawn_cauldron_menu_ui(
    mut commands: Commands,
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
    selection: Res<IngredientSelection>,
    config: Res<GameConfig>,
    stone_used: Res<PhilosophersStoneUsed>,
) {
    let is_brewing = cauldron_query
        .single()
        .is_ok_and(|state| state.is_brewing());

    build_menu(&mut commands, is_brewing, &selection, &config, &stone_used);
}

/// Despawns the menu when the cauldron state changes (e.g. brew completes in urgent mode).
/// `respawn_menu_on_toggle` will rebuild the menu next frame with the updated state.
pub(super) fn rebuild_menu_on_brew_state_change(
    mut commands: Commands,
    cauldron_query: Query<&CauldronState, (With<Cauldron>, Changed<CauldronState>)>,
    menu_query: Query<Entity, With<OnCauldronMenuScreen>>,
) {
    if let Ok(state) = cauldron_query.single()
        && !state.is_brewing()
    {
        for entity in &menu_query {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Re-spawns the menu UI if it was despawned by a toggle action.
pub(super) fn respawn_menu_on_toggle(
    mut commands: Commands,
    menu_query: Query<Entity, With<OnCauldronMenuScreen>>,
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
    selection: Res<IngredientSelection>,
    config: Res<GameConfig>,
    stone_used: Res<PhilosophersStoneUsed>,
) {
    if menu_query.iter().next().is_none() {
        let is_brewing = cauldron_query
            .single()
            .is_ok_and(|state| state.is_brewing());

        build_menu(&mut commands, is_brewing, &selection, &config, &stone_used);
    }
}

/// Builds the cauldron menu UI tree with a two-panel layout.
fn build_menu(
    commands: &mut Commands,
    is_brewing: bool,
    selection: &IngredientSelection,
    config: &GameConfig,
    stone_used: &PhilosophersStoneUsed,
) {
    // Load save data once for unlocked ingredients and combos
    let save = load_unified_save();
    let unlocked_ingredients = save
        .as_ref()
        .map(|s| s.player.unlocked_content.ingredients.clone())
        .unwrap_or_default();
    let unlocked_combos = save
        .as_ref()
        .map(|s| s.player.unlocked_content.combos.clone())
        .unwrap_or_default();

    // Page container (standard overlay with content box). `ModalOverlay`
    // scopes focus to this menu so HUD buttons behind it aren't reachable.
    let content = spawn_page_container(
        commands,
        OnCauldronMenuScreen,
        false,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(LAYOUT_PADDING)),
            row_gap: Val::Px(16.0),
            border: UiRect::all(Val::Px(1.0)),
            overflow: Overflow::clip(),
            ..default()
        },
    );
    commands
        .entity(content)
        .insert(crate::ui::focus::ModalOverlay);

    commands.entity(content).with_children(|root| {
        // Header row: title left, Back button right
        root.spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|header| {
            spawn_title_with_shadow(
                header,
                "Cauldron",
                TITLE_FONT_SIZE,
                TITLE_COLOR,
                Node::default(),
            );
            header.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            if is_brewing {
                spawn_button(
                    header,
                    "Cancel Brew",
                    CauldronMenuButtonAction::CancelBrew,
                    &CANCEL_BUTTON_STYLE,
                );
            }
            spawn_button(
                header,
                "Back",
                (
                    CauldronMenuButtonAction::Close,
                    crate::ui::focus::NoGamepadFocus,
                ),
                &crate::ui::main_menu::BACK_BUTTON_STYLE,
            );
        });

        // Content area: two-panel row (centered when brewing)
        root.spawn(Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(COLUMN_GAP),
            justify_content: if is_brewing {
                JustifyContent::Center
            } else {
                JustifyContent::default()
            },
            align_items: if is_brewing {
                AlignItems::Center
            } else {
                AlignItems::default()
            },
            ..default()
        })
        .with_children(|content| {
            // === Left panel: detail/preview ===
            spawn_detail_panel(
                content,
                is_brewing,
                selection,
                &unlocked_combos,
                config,
                stone_used,
            );

            // === Right panel: categorized ingredient grid ===
            if !is_brewing {
                spawn_ingredient_list(content, selection, &unlocked_ingredients);
            }
        });
    });
}

/// Spawns the left detail panel showing brew preview or brewing status.
fn spawn_detail_panel(
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

/// Spawns the right panel with 4 category columns of ingredients.
fn spawn_ingredient_list(
    parent: &mut ChildSpawnerCommands,
    selection: &IngredientSelection,
    unlocked_ingredients: &[String],
) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(CATEGORY_GAP),
                overflow: Overflow::scroll_y(),
                border: UiRect::all(Val::Px(LIST_BORDER_WIDTH)),
                padding: UiRect::all(Val::Px(LIST_PADDING)),
                border_radius: BorderRadius::all(Val::Px(LIST_BORDER_RADIUS)),
                ..default()
            },
            BackgroundColor(LIST_BG),
            BorderColor::all(LIST_BORDER),
            ScrollPosition::default(),
            crate::ui::focus::GamepadScrollTarget,
        ))
        .with_children(|list| {
            let at_limit = selection.at_limit();

            for category in IngredientCategory::all() {
                // Collect unlocked ingredients in this category
                let mut category_ingredients: Vec<Ingredient> = Ingredient::all()
                    .iter()
                    .copied()
                    .filter(|i| {
                        i.category() == *category && {
                            let debug_name = format!("{:?}", i);
                            unlocked_ingredients.contains(&debug_name)
                        }
                    })
                    .collect();

                if category_ingredients.is_empty() {
                    continue;
                }

                // Sort alphabetically
                category_ingredients.sort_by_key(|i| i.name());

                // Category column
                list.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(LIST_ITEM_GAP),
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|column| {
                    // Category header
                    column.spawn((
                        Text::new(category.display_name()),
                        TextFont::from_font_size(CATEGORY_FONT_SIZE),
                        TextColor(CATEGORY_COLOR),
                        TextLayout::new_with_justify(Justify::Center),
                        Node {
                            width: Val::Percent(100.0),
                            margin: UiRect::bottom(Val::Px(4.0)),
                            ..default()
                        },
                    ));

                    // Ingredient cards
                    for ingredient in &category_ingredients {
                        let is_selected = selection.is_selected(ingredient);
                        spawn_ingredient_card(
                            column,
                            *ingredient,
                            is_selected,
                            at_limit && !is_selected,
                        );
                    }
                });
            }
        });
}

/// Spawns a single ingredient card with toggle button and description.
fn spawn_ingredient_card(
    parent: &mut ChildSpawnerCommands,
    ingredient: Ingredient,
    selected: bool,
    at_limit: bool,
) {
    let button_style = if selected {
        &INGREDIENT_SELECTED_STYLE
    } else if at_limit {
        &INGREDIENT_DISABLED_STYLE
    } else {
        &INGREDIENT_BUTTON_STYLE
    };
    let text_color = if selected {
        INGREDIENT_SELECTED_STYLE.text_color
    } else if at_limit {
        INGREDIENT_DISABLED_STYLE.text_color
    } else {
        TEXT_COLOR
    };

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|card| {
            spawn_button(
                card,
                ingredient.name(),
                (
                    CauldronMenuButtonAction::ToggleIngredient(ingredient),
                    crate::ui::focus::CrossRowHorizontalNav,
                ),
                button_style,
            );

            card.spawn((
                Text::new(ingredient.functional_description()),
                TextFont::from_font_size(DESCRIPTION_FONT_SIZE),
                TextColor(text_color),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    max_width: Val::Px(BUTTON_WIDTH),
                    ..default()
                },
            ));
        });
}

/// Spawns the Philosopher's Stone selector for the Alchemist — a gold toggle
/// button plus its description. Lives at the top of the left detail panel.
/// Spawns nothing when `show` is false (non-Alchemist, already used this battle,
/// or while brewing).
pub(super) fn spawn_philosophers_stone_selector(
    parent: &mut ChildSpawnerCommands,
    has_stone: bool,
    show: bool,
) {
    if !show {
        return;
    }

    let stone_style = if has_stone {
        &STONE_SELECTED_STYLE
    } else {
        &STONE_BUTTON_STYLE
    };

    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                ..default()
            },
            StoneSelectorPanel,
        ))
        .with_children(|card| {
            spawn_button(
                card,
                "Philosopher's Stone",
                (
                    CauldronMenuButtonAction::TogglePhilosophersStone,
                    crate::ui::focus::CrossRowHorizontalNav,
                ),
                stone_style,
            );

            card.spawn((
                Text::new("Removes dilution (once per battle)"),
                TextFont::from_font_size(DESCRIPTION_FONT_SIZE),
                TextColor(stone_style.text_color),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    max_width: Val::Px(BUTTON_WIDTH),
                    ..default()
                },
            ));
        });
}
