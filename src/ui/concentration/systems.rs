use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::styles::*;
use crate::game::components::{ConcentrationSpell, OnGameplayScreen};
use crate::game::input::messages::MouseClicked;

/// Spawns the concentration UI when a concentration spell is active.
pub(super) fn spawn_concentration_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<ConcentrationUIRoot>>,
    concentration_spells: Query<&ConcentrationSpell>,
    mut spell_name_text: Query<&mut Text, With<ConcentrationSpellNameText>>,
) {
    let has_concentration_spell = !concentration_spells.is_empty();
    let has_ui = !ui_query.is_empty();

    // Get the spell name if there is a concentration spell active
    let spell_name = concentration_spells
        .iter()
        .next()
        .map(|spell| spell.spell_name)
        .unwrap_or("Unknown");

    // Spawn UI if concentration spell exists but UI doesn't
    if has_concentration_spell && !has_ui {
        commands
            .spawn((
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(CONCENTRATION_UI_BOTTOM_MARGIN),
                    right: Val::Px(20.0),
                    height: Val::Px(CONCENTRATION_UI_HEIGHT),
                    padding: UiRect::all(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(BUTTON_BACKGROUND),
                BorderColor::all(BUTTON_BORDER),
                BorderRadius::all(Val::Px(4.0)),
                ConcentrationUIRoot,
                EndConcentrationButton,
                OnGameplayScreen,
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new(format!("End Concentration: {}", spell_name)),
                    TextFont {
                        // font removed (using default),
                        font_size: BUTTON_FONT_SIZE,
                        ..default()
                    },
                    TextColor(BUTTON_TEXT_COLOR),
                    ConcentrationSpellNameText,
                ));
            });
    }

    // Despawn UI if no concentration spell but UI exists
    if !has_concentration_spell && has_ui {
        for entity in ui_query.iter() {
            commands.entity(entity).despawn();
        }
    }

    // Update button text if UI exists and spell changed
    if has_ui && has_concentration_spell {
        for mut text in spell_name_text.iter_mut() {
            **text = format!("End Concentration: {}", spell_name);
        }
    }
}

/// Handles hover effects for the "End Concentration" button.
pub(super) fn update_button_hover(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (With<EndConcentrationButton>, Changed<Interaction>),
    >,
) {
    for (interaction, mut background) in button_query.iter_mut() {
        *background = match interaction {
            Interaction::Hovered | Interaction::Pressed => BackgroundColor(BUTTON_HOVER),
            Interaction::None => BackgroundColor(BUTTON_BACKGROUND),
        };
    }
}

/// Handles clicking the "End Concentration" button to cancel concentration spells.
pub(super) fn handle_end_concentration_click(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<Entity, With<EndConcentrationButton>>,
    mut commands: Commands,
    concentration_spells: Query<Entity, With<ConcentrationSpell>>,
) {
    for click in button_clicked.read() {
        if button_query.iter().any(|e| e == click.button) {
            // Despawn all concentration spells
            for spell_entity in concentration_spells.iter() {
                commands.entity(spell_entity).despawn();
            }
        }
    }
}
