use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::game::units::wizard::components::{PrimeSpellMessage, Spell};
use crate::state::InGameState;
use crate::ui::systems::spawn_button;

/// Resource to track when we just entered the spell book.
/// Prevents spell casting on the same frame as opening the spell book.
#[derive(Resource, Default)]
pub struct JustEnteredSpellBook(pub bool);

/// Marker component to track that a button was pressed down.
#[derive(Component)]
pub(super) struct ButtonPressedDown;

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

            // Spell buttons container - grid with max 4 buttons per column
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(MARGIN),
                    ..default()
                })
                .with_children(|grid| {
                    // Split spells into columns of max 4 buttons each
                    let spells = Spell::all();
                    let max_per_column = 4;
                    let columns = spells.len().div_ceil(max_per_column);

                    for col in 0..columns {
                        let start = col * max_per_column;
                        let end = (start + max_per_column).min(spells.len());

                        grid.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(MARGIN),
                            ..default()
                        })
                        .with_children(|column| {
                            for spell in &spells[start..end] {
                                spawn_button(
                                    column,
                                    spell.name(),
                                    SpellBookButtonAction::SelectSpell(*spell),
                                    &BUTTON_STYLE,
                                );
                            }
                        });
                    }
                });

            // Close button in its own row at the bottom
            spawn_button(parent, "Close", SpellBookButtonAction::Close, &BUTTON_STYLE);
        });
}

/// Handles button click actions and sends prime spell messages.
/// Uses a marker component to ensure buttons only trigger on release after being pressed.
pub fn button_action(
    mut commands: Commands,
    interaction_query: Query<
        (
            Entity,
            &Interaction,
            &SpellBookButtonAction,
            Option<&ButtonPressedDown>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut prime_spell: MessageWriter<PrimeSpellMessage>,
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
                        SpellBookButtonAction::SelectSpell(spell) => {
                            prime_spell.write(PrimeSpellMessage {
                                spell: spell.primed_config(),
                            });
                            next_in_game_state.set(InGameState::Running);
                        }
                        SpellBookButtonAction::Close => {
                            next_in_game_state.set(InGameState::Running);
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

/// Sets the flag when entering spell book to prevent spell casting.
pub fn set_just_entered_flag(mut just_entered: ResMut<JustEnteredSpellBook>) {
    just_entered.0 = true;
}

/// Clears the flag after one frame in SpellBook state.
pub fn clear_just_entered_flag(mut just_entered: ResMut<JustEnteredSpellBook>) {
    just_entered.0 = false;
}
