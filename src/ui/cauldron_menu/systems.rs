use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::config::save_data::load_unified_save;
use crate::game::cauldron::brews::{BrewEffect, Ingredient, Recipe};
use crate::game::cauldron::components::{Cauldron, CauldronState};
use crate::game::cauldron::messages::{CancelBrewMessage, StartBrewMessage};
use crate::game::input::messages::MouseClicked;
use crate::state::InGameState;
use crate::ui::systems::spawn_button;

/// Spawns the cauldron menu UI when entering the CauldronMenu state.
pub(super) fn spawn_cauldron_menu_ui(
    mut commands: Commands,
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
    selection: Res<IngredientSelection>,
) {
    let is_brewing = cauldron_query
        .single()
        .is_ok_and(|state| state.is_brewing());

    build_menu(&mut commands, is_brewing, &selection);
}

/// Re-spawns the menu UI if it was despawned by a toggle action.
pub(super) fn respawn_menu_on_toggle(
    mut commands: Commands,
    menu_query: Query<Entity, With<OnCauldronMenuScreen>>,
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
    selection: Res<IngredientSelection>,
) {
    if menu_query.iter().next().is_none() {
        let is_brewing = cauldron_query
            .single()
            .is_ok_and(|state| state.is_brewing());

        build_menu(&mut commands, is_brewing, &selection);
    }
}

/// Builds the cauldron menu UI tree.
fn build_menu(commands: &mut Commands, is_brewing: bool, selection: &IngredientSelection) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(MARGIN),
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            OnCauldronMenuScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Cauldron"),
                TextFont::from_font_size(TITLE_FONT_SIZE),
                TextColor(TEXT_COLOR),
            ));

            if is_brewing {
                // Show brewing status
                parent.spawn((
                    Text::new("Brewing in progress..."),
                    TextFont::from_font_size(BREWING_STATUS_FONT_SIZE),
                    TextColor(BREWING_STATUS_COLOR),
                ));

                // Cancel button
                spawn_button(
                    parent,
                    "Cancel Brew",
                    CauldronMenuButtonAction::CancelBrew,
                    &CANCEL_BUTTON_STYLE,
                );
            } else {
                // Ingredient selection frame
                parent
                    .spawn((
                        Node {
                            border: UiRect::all(Val::Px(FRAME_BORDER_WIDTH)),
                            padding: UiRect::all(Val::Px(FRAME_PADDING)),
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(INGREDIENT_COLUMN_GAP),
                            row_gap: Val::Px(INGREDIENT_COLUMN_GAP),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::FlexStart,
                            max_width: Val::Vw(90.0),
                            ..default()
                        },
                        BorderColor::all(FRAME_BORDER_COLOR),
                        BorderRadius::all(Val::Px(8.0)),
                        BackgroundColor(FRAME_BACKGROUND),
                    ))
                    .with_children(|container| {
                        // Load save data to get unlocked ingredients
                        let save = load_unified_save();
                        let unlocked_ingredients = save
                            .as_ref()
                            .map(|s| s.player.unlocked_content.ingredients.clone())
                            .unwrap_or_default();

                        for ingredient in Ingredient::all() {
                            let debug_name = format!("{:?}", ingredient);
                            if unlocked_ingredients.contains(&debug_name) {
                                spawn_ingredient_card(
                                    container,
                                    *ingredient,
                                    selection.is_selected(ingredient),
                                );
                            }
                        }
                    });

                // Effect preview (only when something is selected)
                if !selection.is_empty() {
                    let recipe = Recipe::new(selection.selected.clone());
                    spawn_effect_preview(parent, &recipe);
                }

                // Brew button
                let brew_style = if selection.is_empty() {
                    &BREW_BUTTON_DISABLED_STYLE
                } else {
                    &BREW_BUTTON_STYLE
                };
                spawn_button(
                    parent,
                    "Brew",
                    CauldronMenuButtonAction::StartBrew,
                    brew_style,
                );
            }

            // Close button
            spawn_button(
                parent,
                "Close",
                CauldronMenuButtonAction::Close,
                &CLOSE_BUTTON_STYLE,
            );
        });
}

/// Spawns a single ingredient card with toggle button and description.
fn spawn_ingredient_card(
    parent: &mut ChildSpawnerCommands,
    ingredient: Ingredient,
    selected: bool,
) {
    let button_style = if selected {
        &INGREDIENT_SELECTED_STYLE
    } else {
        &INGREDIENT_BUTTON_STYLE
    };
    let text_color = if selected {
        INGREDIENT_SELECTED_STYLE.text_color
    } else {
        TEXT_COLOR
    };

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(8.0),
            width: Val::Px(INGREDIENT_COLUMN_WIDTH),
            padding: UiRect::horizontal(Val::Px(COLUMN_PADDING)),
            ..default()
        })
        .with_children(|card| {
            spawn_button(
                card,
                ingredient.name(),
                CauldronMenuButtonAction::ToggleIngredient(ingredient),
                button_style,
            );

            card.spawn((
                Text::new(ingredient.description()),
                TextFont::from_font_size(DESCRIPTION_FONT_SIZE),
                TextColor(text_color),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

/// Spawns the effect preview section showing computed effects.
fn spawn_effect_preview(parent: &mut ChildSpawnerCommands, recipe: &Recipe) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|preview| {
            // Brew time info
            preview.spawn((
                Text::new(format!(
                    "Brew time: {:.0}s | Buff duration: {:.0}s",
                    recipe.brew_time(),
                    recipe.buff_duration()
                )),
                TextFont::from_font_size(BREW_INFO_FONT_SIZE),
                TextColor(BREW_INFO_COLOR),
            ));

            // Effect list
            for effect in &recipe.effects() {
                let text = match effect {
                    BrewEffect::ManaRegenMultiplier(v) => {
                        format!("Mana regen: +{:.0}%", (v - 1.0) * 100.0)
                    }
                    BrewEffect::SpellPowerMultiplier(v) => {
                        format!("Spell power: +{:.0}%", (v - 1.0) * 100.0)
                    }
                    BrewEffect::DefenderHealPerSecond(v) => {
                        format!("Defender healing: {:.1} HP/s", v)
                    }
                    BrewEffect::CastSpeedMultiplier(v) => {
                        format!("Cast speed: +{:.0}%", (v - 1.0) * 100.0)
                    }
                    BrewEffect::SpellRangeMultiplier(v) => {
                        format!("Spell area: +{:.0}%", (v - 1.0) * 100.0)
                    }
                    BrewEffect::DefenderDamageBonus(v) => {
                        format!("Defender damage: +{:.0}%", v * 100.0)
                    }
                    BrewEffect::DamageResistancePercent(v) => {
                        format!("Damage resistance: {:.0}%", v * 100.0)
                    }
                    BrewEffect::DefenderSpeedBonus(v) => {
                        format!("Defender speed: +{:.0}%", v * 100.0)
                    }
                    BrewEffect::AttackerSlowPercent(v) => {
                        format!("Enemy slow: {:.0}%", v * 100.0)
                    }
                    BrewEffect::DefenderShieldPerSecond(v) => {
                        format!("Defender shield: {:.1} HP/s", v)
                    }
                };
                preview.spawn((
                    Text::new(text),
                    TextFont::from_font_size(EFFECT_PREVIEW_FONT_SIZE),
                    TextColor(EFFECT_PREVIEW_COLOR),
                ));
            }

            // Dilution warning if more than 1 ingredient
            if recipe.ingredients.len() > 1 {
                preview.spawn((
                    Text::new(format!(
                        "Dilution: {:.0}% strength per ingredient",
                        recipe.dilution_factor() * 100.0
                    )),
                    TextFont::from_font_size(EFFECT_PREVIEW_FONT_SIZE),
                    TextColor(DISABLED_TEXT_COLOR),
                ));
            }
        });
}

/// Handles button click actions — toggles ingredients, starts brew, cancels, or closes.
#[allow(clippy::too_many_arguments)]
pub(super) fn button_action(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&CauldronMenuButtonAction>,
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
    mut selection: ResMut<IngredientSelection>,
    mut start_brew: MessageWriter<StartBrewMessage>,
    mut cancel_brew: MessageWriter<CancelBrewMessage>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
    menu_query: Query<Entity, With<OnCauldronMenuScreen>>,
) {
    let is_brewing = cauldron_query
        .single()
        .is_ok_and(|state| state.is_brewing());

    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                CauldronMenuButtonAction::ToggleIngredient(ingredient) => {
                    selection.toggle(*ingredient);
                    // Despawn menu so respawn_menu_on_toggle rebuilds it next frame
                    for entity in &menu_query {
                        commands.entity(entity).try_despawn();
                    }
                }
                CauldronMenuButtonAction::StartBrew => {
                    if !is_brewing && !selection.is_empty() {
                        let recipe = Recipe::new(selection.selected.clone());
                        start_brew.write(StartBrewMessage { recipe });
                        selection.clear();
                        next_in_game_state.set(InGameState::Running);
                    }
                }
                CauldronMenuButtonAction::CancelBrew => {
                    cancel_brew.write(CancelBrewMessage);
                    next_in_game_state.set(InGameState::Running);
                }
                CauldronMenuButtonAction::Close => {
                    next_in_game_state.set(InGameState::Running);
                }
            }
        }
    }
}

/// Despawns cauldron menu UI when exiting the CauldronMenu state.
pub(super) fn despawn_cauldron_menu_ui(
    mut commands: Commands,
    query: Query<Entity, With<OnCauldronMenuScreen>>,
) {
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
}

