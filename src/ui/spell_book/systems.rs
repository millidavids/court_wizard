use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::game::units::wizard::components::{PrimedSpell, Wizard};
use crate::game::units::wizard::spells::{
    disintegrate_constants, fireball_constants, magic_missile_constants,
};
use crate::state::InGameState;
use crate::ui::systems::spawn_button;

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
                        &BUTTON_STYLE,
                    );
                    spawn_button(
                        buttons,
                        "Disintegrate",
                        SpellBookButtonAction::Disintegrate,
                        &BUTTON_STYLE,
                    );
                    spawn_button(
                        buttons,
                        "Fireball",
                        SpellBookButtonAction::Fireball,
                        &BUTTON_STYLE,
                    );
                    spawn_button(
                        buttons,
                        "Close",
                        SpellBookButtonAction::Close,
                        &BUTTON_STYLE,
                    );
                });
        });
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
                        *primed_spell = magic_missile_constants::PRIMED_MAGIC_MISSILE;
                    }
                    next_in_game_state.set(InGameState::Running);
                }
                SpellBookButtonAction::Disintegrate => {
                    if let Ok(mut primed_spell) = wizard_query.single_mut() {
                        *primed_spell = disintegrate_constants::PRIMED_DISINTEGRATE;
                    }
                    next_in_game_state.set(InGameState::Running);
                }
                SpellBookButtonAction::Fireball => {
                    if let Ok(mut primed_spell) = wizard_query.single_mut() {
                        *primed_spell = fireball_constants::PRIMED_FIREBALL;
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
