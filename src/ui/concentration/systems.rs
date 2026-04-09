use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::game::components::{ConcentrationSpell, OnGameplayScreen};
use crate::game::input::messages::MouseClicked;

/// Manages the concentration UI: spawns/despawns the root container and
/// individual per-spell cancel buttons as concentration spells come and go.
pub(super) fn spawn_concentration_ui(
    mut commands: Commands,
    ui_root_query: Query<Entity, With<ConcentrationUIRoot>>,
    existing_buttons: Query<(Entity, &ConcentrationSpellButton)>,
    concentration_spells: Query<(Entity, &ConcentrationSpell)>,
) {
    let has_spells = !concentration_spells.is_empty();
    let has_ui = !ui_root_query.is_empty();

    // Despawn root if no concentration spells remain
    if !has_spells && has_ui {
        for entity in ui_root_query.iter() {
            commands.entity(entity).try_despawn();
        }
        return;
    }

    if !has_spells {
        return;
    }

    // Spawn root container if it doesn't exist
    let root = if !has_ui {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(CONCENTRATION_UI_BOTTOM_MARGIN),
                    right: Val::Px(20.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    align_items: AlignItems::FlexEnd,
                    ..default()
                },
                ConcentrationUIRoot,
                OnGameplayScreen,
            ))
            .id()
    } else {
        ui_root_query.iter().next().expect("UI root exists")
    };

    // Collect which spell entities already have buttons
    let existing_spell_entities: Vec<Entity> = existing_buttons
        .iter()
        .map(|(_, btn)| btn.spell_entity)
        .collect();

    // Despawn buttons for spells that no longer exist
    for (button_entity, btn) in existing_buttons.iter() {
        if !concentration_spells.iter().any(|(e, _)| e == btn.spell_entity) {
            commands.entity(button_entity).try_despawn();
        }
    }

    // Spawn buttons for new concentration spells
    for (spell_entity, spell) in concentration_spells.iter() {
        if existing_spell_entities.contains(&spell_entity) {
            continue;
        }

        let button = commands
            .spawn((
                Button,
                Node {
                    height: Val::Px(CONCENTRATION_UI_HEIGHT),
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(BUTTON_BACKGROUND),
                BorderColor::all(BUTTON_BORDER),
                BorderRadius::all(Val::Px(4.0)),
                ConcentrationSpellButton { spell_entity },
            ))
            .with_children(|button_node| {
                button_node.spawn((
                    Text::new(format!(
                        "End: {} [{}]",
                        spell.spell_name, spell.mana_cost as u32
                    )),
                    TextFont::from_font_size(BUTTON_FONT_SIZE),
                    TextColor(BUTTON_TEXT_COLOR),
                ));
            })
            .id();

        commands.entity(root).add_child(button);
    }
}

/// Handles hover effects for concentration spell buttons.
pub(super) fn update_button_hover(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (With<ConcentrationSpellButton>, Changed<Interaction>),
    >,
) {
    for (interaction, mut background) in button_query.iter_mut() {
        *background = match interaction {
            Interaction::Hovered | Interaction::Pressed => BackgroundColor(BUTTON_HOVER),
            Interaction::None => BackgroundColor(BUTTON_BACKGROUND),
        };
    }
}

/// Handles clicking a concentration spell button to cancel only that spell.
pub(super) fn handle_end_concentration_click(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<(Entity, &ConcentrationSpellButton)>,
    mut commands: Commands,
) {
    for click in button_clicked.read() {
        for (button_entity, btn) in button_query.iter() {
            if button_entity == click.button {
                // Despawn the concentration spell entity (ends the spell)
                commands.entity(btn.spell_entity).try_despawn();
                // Despawn the button itself
                commands.entity(button_entity).try_despawn();
                break;
            }
        }
    }
}
