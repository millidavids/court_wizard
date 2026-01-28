//! In-game systems for input handling and HUD management.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::input::events::BlockSpellInput;
use crate::game::resources::CurrentLevel;
use crate::game::units::wizard::components::{CastingState, Mana, PrimedSpell, Wizard};
use crate::state::InGameState;
use crate::ui::systems::spawn_button;

/// Marker component to track that a button was pressed down.
#[derive(Component)]
pub(super) struct ButtonPressedDown;

/// Blocks spell input when any button is being interacted with.
///
/// This system runs before spell systems to prevent casting when clicking UI buttons.
pub fn block_spell_input_on_button_interaction(
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
pub fn keyboard_input(
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
pub fn spawn_hud(
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    config: Res<GameConfig>,
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
                    // Spell book button (top-left)
                    spawn_button(row, "Spells", HudButtonAction::OpenSpellBook, &BUTTON_STYLE);

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
                    });
                });
        });
}

/// Handles HUD button click actions.
///
/// Uses a marker component to ensure buttons only trigger on release after being pressed.
pub fn hud_button_action(
    mut commands: Commands,
    interaction_query: Query<
        (
            Entity,
            &Interaction,
            &HudButtonAction,
            Option<&ButtonPressedDown>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    for (entity, interaction, action, pressed_down) in &interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Mark button as pressed down
                commands.entity(entity).insert(ButtonPressedDown);
            }
            Interaction::Hovered => {
                // Only trigger action if button was previously pressed
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();

                    match action {
                        HudButtonAction::OpenSpellBook => {
                            next_in_game_state.set(InGameState::SpellBook);
                        }
                    }
                }
            }
            Interaction::None => {
                // Clear marker if mouse leaves button
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();
                }
            }
        }
    }
}

/// Updates the mana bar width based on current wizard mana.
pub fn update_mana_bar(
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

/// Updates the cast bar width based on current wizard casting progress.
///
/// Uses the cast time from the currently primed spell.
pub fn update_cast_bar(
    wizard_query: Query<(&CastingState, &PrimedSpell), With<Wizard>>,
    mut cast_bar_query: Query<&mut Node, With<CastBarFill>>,
) {
    if let Ok((casting_state, primed_spell)) = wizard_query.single()
        && let Ok(mut node) = cast_bar_query.single_mut()
    {
        let progress_percent = casting_state.progress(primed_spell.cast_time) * 100.0;
        node.width = Val::Percent(progress_percent);
    }
}

/// Updates the level display text when the current level changes.
pub fn update_level_display(
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
pub fn update_past_victory_display(
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
