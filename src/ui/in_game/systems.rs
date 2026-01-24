//! In-game systems for input handling and HUD management.

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use super::components::*;
use super::styles::*;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::components::{CastingState, Mana, Wizard};
use crate::state::InGameState;

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
/// - Mana bar in bottom right corner
/// - Cast bar below mana bar
pub fn spawn_hud(mut commands: Commands) {
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
            // Spell book button (top-left)
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::FlexStart,
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(BUTTON_WIDTH),
                            height: Val::Px(BUTTON_HEIGHT),
                            border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor::all(BUTTON_BORDER),
                        BorderRadius::all(Val::Px(8.0)),
                        BackgroundColor(BUTTON_BACKGROUND),
                        ButtonColors {
                            background: BUTTON_BACKGROUND,
                            border: BUTTON_BORDER,
                        },
                        HudButtonAction::OpenSpellBook,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("Spells"),
                            TextFont {
                                font_size: BUTTON_FONT_SIZE,
                                ..default()
                            },
                            TextColor(BUTTON_TEXT_COLOR),
                        ));
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

/// Handles HUD button visual states (hover/pressed).
pub fn hud_button_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &ButtonColors,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, colors, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                use crate::ui::styles::item_pressed;
                *bg_color = item_pressed(colors.background).into();
                *border_color = BorderColor::all(item_pressed(colors.border));
            }
            Interaction::Hovered => {
                use crate::ui::styles::item_hovered;
                *bg_color = item_hovered(colors.background).into();
                *border_color = BorderColor::all(item_hovered(colors.border));
            }
            Interaction::None => {
                *bg_color = colors.background.into();
                *border_color = BorderColor::all(colors.border);
            }
        }
    }
}

/// Handles HUD button click actions.
pub fn hud_button_action(
    interaction_query: Query<
        (&Interaction, &HudButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                HudButtonAction::OpenSpellBook => {
                    next_in_game_state.set(InGameState::SpellBook);
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
/// Cast time is currently hardcoded to match magic missile (1 second).
pub fn update_cast_bar(
    wizard_query: Query<&CastingState, With<Wizard>>,
    mut cast_bar_query: Query<&mut Node, With<CastBarFill>>,
) {
    if let Ok(casting_state) = wizard_query.single()
        && let Ok(mut node) = cast_bar_query.single_mut()
    {
        // Magic missile cast time
        const CAST_TIME: f32 = 1.0;

        let progress_percent = casting_state.progress(CAST_TIME) * 100.0;
        node.width = Val::Percent(progress_percent);
    }
}
