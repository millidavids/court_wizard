//! In-game systems for input handling and HUD management.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::config::GameConfig;
use crate::game::cauldron::components::{Cauldron, CauldronState};
use crate::game::components::OnGameplayScreen;
use crate::game::input::messages::{BlockSpellInput, MouseClicked};
use crate::game::resources::CurrentLevel;
use crate::game::units::wizard::components::{CastingState, Mana, PrimedSpell, Wizard};
use crate::state::InGameState;
use crate::ui::resources::CustomFont;
use crate::ui::systems::spawn_button;

/// Blocks spell input when any button is being interacted with.
///
/// This system runs before spell systems to prevent casting when clicking UI buttons.
pub(super) fn block_spell_input_on_button_interaction(
    button_query: Query<&Interaction, With<Button>>,
    mut block_spell_input: MessageWriter<BlockSpellInput>,
) {
    // Block spell input if any button is pressed or hovered
    for interaction in &button_query {
        if matches!(*interaction, Interaction::Pressed | Interaction::Hovered) {
            block_spell_input.write(BlockSpellInput);
            return; // Only need to send once
        }
    }
}

/// Handles keyboard input during active gameplay.
///
/// - Escape: Pause the game, transitioning to `InGameState::Paused`
pub(super) fn keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_in_game_state.set(InGameState::Paused);
    }
}

/// Spawns the gameplay HUD.
///
/// Creates a HUD with margins around screen edges containing:
/// - Spell book button in top left corner
/// - Level indicator and past victory in top right corner
/// - Mana bar in bottom right corner
/// - Cast bar below mana bar
pub(super) fn spawn_hud(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    config: Res<GameConfig>,
    custom_font: Res<CustomFont>,
) {
    // Root HUD container (fullscreen with margins)
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(HUD_MARGIN),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            HudRoot,
            OnGameplayScreen,
        ))
        .with_children(|parent| {
            // Top row (spell book button on left, level on right)
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexStart,
                    ..default()
                })
                .with_children(|row| {
                    // Button group (top-left)
                    row.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|buttons| {
                        spawn_button(
                            buttons,
                            "Spells",
                            HudButtonAction::OpenSpellBook,
                            &BUTTON_STYLE,
                            &custom_font,
                        );
                        spawn_button(
                            buttons,
                            "Cauldron",
                            HudButtonAction::OpenCauldronMenu,
                            &BUTTON_STYLE,
                            &custom_font,
                        );
                    });

                    // Level and past victory display (top-right)
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexEnd,
                        row_gap: Val::Px(5.0),
                        ..default()
                    })
                    .with_children(|level_container| {
                        // Level display
                        level_container.spawn((
                            Text::new(format!("Level: {}", current_level.0)),
                            TextFont {
                                font: custom_font.handle.clone(),
                                font_size: 30.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            LevelDisplay,
                        ));

                        // Past victory display (if exists)
                        if let Some(past_efficiency) =
                            config.efficiency_ratios.get(&current_level.0.to_string())
                        {
                            level_container.spawn((
                                Text::new(format!("Best: {:.1}%", past_efficiency * 100.0)),
                                TextFont {
                                    font: custom_font.handle.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.8, 0.8, 0.8, 0.9)),
                                PastVictoryDisplay,
                            ));
                        } else {
                            // Spawn empty placeholder so the component exists
                            level_container.spawn((
                                Text::new(""),
                                TextFont {
                                    font: custom_font.handle.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.8, 0.8, 0.8, 0.9)),
                                PastVictoryDisplay,
                            ));
                        }
                    });
                });

            // Bottom-right bars container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    row_gap: HUD_ELEMENT_GAP,
                    ..default()
                })
                .with_children(|bars| {
                    // Mana bar container (background)
                    bars.spawn((
                        Node {
                            width: MANA_BAR_WIDTH,
                            height: MANA_BAR_HEIGHT,
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::FlexEnd, // Fill from right, empties from left
                            ..default()
                        },
                        BackgroundColor(MANA_BAR_BG_COLOR),
                    ))
                    .with_children(|parent| {
                        // Mana bar fill (starts at 100%, reduces from left)
                        parent.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(MANA_BAR_FILL_COLOR),
                            ManaBarFill,
                        ));
                    });

                    // Cast bar container (background)
                    bars.spawn((
                        Node {
                            width: CAST_BAR_WIDTH,
                            height: CAST_BAR_HEIGHT,
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::FlexEnd, // Fill from right
                            ..default()
                        },
                        BackgroundColor(CAST_BAR_BG_COLOR),
                    ))
                    .with_children(|cast_bar| {
                        // Cast bar fill (starts at 0%)
                        cast_bar.spawn((
                            Node {
                                width: Val::Percent(0.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(CAST_BAR_FILL_COLOR),
                            CastBarFill,
                        ));

                        // Brewing overlay container (hidden by default, shown during brewing)
                        cast_bar
                            .spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                Visibility::Hidden,
                                BrewingOverlay,
                            ))
                            .with_children(|overlay| {
                                overlay.spawn((
                                    Text::new("Brewing..."),
                                    TextFont {
                                        font: custom_font.handle.clone(),
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    });
                });
        });
}

/// Handles HUD button click actions.
pub(super) fn hud_button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&HudButtonAction>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                HudButtonAction::OpenSpellBook => {
                    next_in_game_state.set(InGameState::SpellBook);
                }
                HudButtonAction::OpenCauldronMenu => {
                    next_in_game_state.set(InGameState::CauldronMenu);
                }
            }
        }
    }
}

/// Updates the mana bar width based on current wizard mana.
pub(super) fn update_mana_bar(
    wizard_query: Query<&Mana, With<Wizard>>,
    mut mana_bar_query: Query<&mut Node, With<ManaBarFill>>,
) {
    if let Ok(mana) = wizard_query.single()
        && let Ok(mut node) = mana_bar_query.single_mut()
    {
        let mana_percent = mana.percentage() * 100.0;
        node.width = Val::Percent(mana_percent);
    }
}

/// Updates the cast bar width based on current wizard casting progress or brewing progress.
///
/// When the cauldron is brewing:
/// - Shows brewing progress with a grayed-out fill
/// - Displays "Brewing..." text overlay
///
/// Otherwise shows normal cast progress with gold fill.
pub(super) fn update_cast_bar(
    wizard_query: Query<(&CastingState, &PrimedSpell), With<Wizard>>,
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
    mut cast_bar_query: Query<(&mut Node, &mut BackgroundColor), With<CastBarFill>>,
    mut overlay_query: Query<&mut Visibility, With<BrewingOverlay>>,
) {
    let is_brewing = cauldron_query
        .single()
        .is_ok_and(|state| state.is_brewing());

    if let Ok((mut node, mut bg_color)) = cast_bar_query.single_mut() {
        if is_brewing {
            // Show brewing progress with gray fill
            if let Ok(state) = cauldron_query.single() {
                let progress_percent = state.progress() * 100.0;
                node.width = Val::Percent(progress_percent);
            }
            bg_color.0 = CAST_BAR_BREWING_FILL_COLOR;
        } else {
            // Show normal cast progress with gold fill
            if let Ok((casting_state, primed_spell)) = wizard_query.single() {
                let progress_percent = casting_state.progress(primed_spell.cast_time) * 100.0;
                node.width = Val::Percent(progress_percent);
            }
            bg_color.0 = CAST_BAR_FILL_COLOR;
        }
    }

    // Toggle brewing overlay visibility
    if let Ok(mut visibility) = overlay_query.single_mut() {
        *visibility = if is_brewing {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Updates the level display text when the current level changes.
pub(super) fn update_level_display(
    current_level: Res<CurrentLevel>,
    mut level_display_query: Query<&mut Text, With<LevelDisplay>>,
) {
    if current_level.is_changed()
        && let Ok(mut text) = level_display_query.single_mut()
    {
        **text = format!("Level: {}", current_level.0);
    }
}

/// Updates the past victory display text when the current level changes.
pub(super) fn update_past_victory_display(
    current_level: Res<CurrentLevel>,
    config: Res<GameConfig>,
    mut past_victory_query: Query<&mut Text, With<PastVictoryDisplay>>,
) {
    if current_level.is_changed()
        && let Ok(mut text) = past_victory_query.single_mut()
    {
        if let Some(past_efficiency) = config.efficiency_ratios.get(&current_level.0.to_string()) {
            **text = format!("Best: {:.1}%", past_efficiency * 100.0);
        } else {
            **text = String::new();
        }
    }
}
