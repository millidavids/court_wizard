use bevy::prelude::*;

use crate::game::cauldron::brews::{Ingredient, IngredientCategory};
use crate::ui::cauldron_menu::components::*;
use crate::ui::cauldron_menu::constants::*;
use crate::ui::systems::spawn_button;

/// Spawns the right panel with 4 category columns of ingredients.
pub(super) fn spawn_ingredient_list(
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
                            let debug_name = i.save_key().to_string();
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
