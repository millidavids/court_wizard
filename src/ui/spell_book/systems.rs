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
                    // Generate a button for each spell
                    for spell in Spell::all() {
                        spawn_button(
                            buttons,
                            spell.name(),
                            SpellBookButtonAction::SelectSpell(*spell),
                            &BUTTON_STYLE,
                        );
                    }

                    spawn_button(
                        buttons,
                        "Close",
                        SpellBookButtonAction::Close,
                        &BUTTON_STYLE,
                    );
                });
        });
}

/// Handles button click actions and sends prime spell messages.
pub fn button_action(
    interaction_query: Query<
        (&Interaction, &SpellBookButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut prime_spell: MessageWriter<PrimeSpellMessage>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    for (interaction, action) in &interaction_query {
        if *interaction == Interaction::Pressed {
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
