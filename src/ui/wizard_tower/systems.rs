use bevy::prelude::*;

use crate::config::ActiveSave;
use crate::config::save_data::{load_unified_save, unlock_ingredient};
use crate::game::cauldron::brews::Ingredient;
use crate::game::input::messages::MouseClicked;
use crate::game::resources::{CurrentLevel, KillStats};
use crate::state::AppState;
use crate::ui::systems::spawn_button;

use super::components::*;
use super::constants::*;

/// Attempts to randomly unlock a locked ingredient when entering wizard tower.
pub(super) fn try_unlock_random_ingredient(mut newly_unlocked: ResMut<NewlyUnlockedIngredient>) {
    use rand::seq::SliceRandom;

    // Load current unlocked ingredients
    let save = load_unified_save();
    let unlocked_ingredients: Vec<String> = save
        .map(|s| s.player.unlocked_content.ingredients)
        .unwrap_or_default();

    // Find locked ingredients
    let locked: Vec<Ingredient> = Ingredient::all()
        .iter()
        .filter(|ingredient| {
            let debug_name = format!("{:?}", ingredient);
            !unlocked_ingredients.contains(&debug_name)
        })
        .copied()
        .collect();

    // If there are locked ingredients, pick one randomly
    if let Some(ingredient) = locked.choose(&mut rand::thread_rng()) {
        if unlock_ingredient(*ingredient) {
            newly_unlocked.0 = Some(*ingredient);
        }
    } else {
        newly_unlocked.0 = None;
    }
}

/// Sets up the wizard tower screen UI.
pub(super) fn setup_wizard_tower_screen(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    newly_unlocked: Res<NewlyUnlockedIngredient>,
) {
    // Root container (fullscreen, centered column layout)
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            OnWizardTowerScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Wizard's Tower"),
                TextFont {
                    // font removed (using default),
                    font_size: TITLE_FONT_SIZE,
                    ..default()
                },
                TextColor(TITLE_COLOR),
            ));

            // Current level display
            parent.spawn((
                Text::new(format!("Level {}", current_level.0)),
                TextFont {
                    // font removed (using default),
                    font_size: LEVEL_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Display newly unlocked ingredient (if any)
            if let Some(ingredient) = newly_unlocked.0 {
                parent
                    .spawn((
                        Node {
                            padding: UiRect::all(Val::Px(20.0)),
                            margin: UiRect::bottom(Val::Px(20.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(10.0),
                            ..default()
                        },
                        BorderColor::all(Color::srgb(0.4, 0.9, 0.4)),
                        BackgroundColor(Color::srgb(0.1, 0.2, 0.1)),
                        BorderRadius::all(Val::Px(8.0)),
                    ))
                    .with_children(|unlock_box| {
                        unlock_box.spawn((
                            Text::new("New Ingredient Discovered!"),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.4, 0.9, 0.4)),
                        ));

                        unlock_box.spawn((
                            Text::new(ingredient.name()),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        ));

                        unlock_box.spawn((
                            Text::new(ingredient.description()),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        ));
                    });
            }

            // Start Next Battle button
            spawn_button(
                parent,
                "Start Next Battle",
                WizardTowerButtonAction::StartNextBattle,
                &BUTTON_STYLE,
            );

            // Return to Menu button
            spawn_button(
                parent,
                "Return to Menu",
                WizardTowerButtonAction::ReturnToMenu,
                &BUTTON_STYLE,
            );
        });
}

/// Handles button actions on the wizard tower screen.
pub(super) fn handle_button_actions(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&WizardTowerButtonAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut kill_stats: ResMut<KillStats>,
    mut active_save: ResMut<ActiveSave>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                WizardTowerButtonAction::StartNextBattle => {
                    // Reset stats and transition to Loading
                    // Level was already updated by GameOver
                    kill_stats.reset();
                    next_app_state.set(AppState::Loading);
                }
                WizardTowerButtonAction::ReturnToMenu => {
                    // Reset stats, clear active save, and return to main menu
                    kill_stats.reset();
                    active_save.0 = None;
                    next_app_state.set(AppState::MainMenu);
                }
            }
        }
    }
}

/// Cleans up the wizard tower screen UI.
pub(super) fn cleanup_wizard_tower_screen(
    mut commands: Commands,
    query: Query<Entity, With<OnWizardTowerScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
