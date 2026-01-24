use bevy::prelude::*;

use super::components::*;
use super::styles::*;
use crate::game::units::wizard::components::{PrimedSpell, SpellType, Wizard};
use crate::state::InGameState;
use crate::ui::styles::{item_hovered, item_pressed};

/// Spawns the spell book UI when entering the SpellBook state.
pub fn spawn_spell_book_ui(mut commands: Commands) {
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
            OnSpellBookScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Select Spell"),
                TextFont {
                    font_size: TITLE_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));

            // Spell buttons container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(MARGIN),
                    ..default()
                })
                .with_children(|buttons| {
                    spawn_button(
                        buttons,
                        "Magic Missile",
                        SpellBookButtonAction::MagicMissile,
                    );
                    spawn_button(buttons, "Disintegrate", SpellBookButtonAction::Disintegrate);
                    spawn_button(buttons, "Close", SpellBookButtonAction::Close);
                });
        });
}

/// Spawns a single button with the given text and action.
fn spawn_button(parent: &mut ChildSpawnerCommands, text: &str, action: SpellBookButtonAction) {
    parent
        .spawn((
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
            action,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(text),
                TextFont {
                    font_size: BUTTON_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
        });
}

/// Handles button visual states (hover/pressed).
pub fn button_interaction(
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
                *bg_color = item_pressed(colors.background).into();
                *border_color = BorderColor::all(item_pressed(colors.border));
            }
            Interaction::Hovered => {
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

/// Handles button click actions.
pub fn button_action(
    interaction_query: Query<
        (&Interaction, &SpellBookButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut wizard_query: Query<&mut PrimedSpell, With<Wizard>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match action {
                SpellBookButtonAction::MagicMissile => {
                    if let Ok(mut primed_spell) = wizard_query.single_mut() {
                        primed_spell.spell = SpellType::MagicMissile;
                    }
                    next_in_game_state.set(InGameState::Running);
                }
                SpellBookButtonAction::Disintegrate => {
                    if let Ok(mut primed_spell) = wizard_query.single_mut() {
                        primed_spell.spell = SpellType::Disintegrate;
                    }
                    next_in_game_state.set(InGameState::Running);
                }
                SpellBookButtonAction::Close => {
                    next_in_game_state.set(InGameState::Running);
                }
            }
        }
    }
}

/// Handles keyboard input (ESC to close).
pub fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_in_game_state.set(InGameState::Running);
    }
}

/// Despawns spell book UI when exiting the SpellBook state.
pub fn despawn_spell_book_ui(
    mut commands: Commands,
    query: Query<Entity, With<OnSpellBookScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
