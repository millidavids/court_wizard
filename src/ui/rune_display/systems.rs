use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::input::events::MouseClicked;
use crate::game::runes::RuneSequence;
use crate::game::runes::constants::sequence_to_spell;
use crate::game::runes::resources::Rune;
use crate::ui::components::ButtonColors;

/// Spawns the rune display UI with 4 clickable buttons and sequence text above.
pub(super) fn spawn_rune_display(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(BOTTOM_MARGIN),
                left: Val::Px(LEFT_MARGIN),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                ..default()
            },
            RuneDisplayRoot,
            OnGameplayScreen,
        ))
        .with_children(|parent| {
            // Sequence text above buttons
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: RUNE_SEQUENCE_FONT_SIZE,
                    ..default()
                },
                TextColor(SEQUENCE_TEXT_COLOR),
                Node {
                    min_height: Val::Px(RUNE_SEQUENCE_FONT_SIZE + 4.0),
                    ..default()
                },
                RuneSequenceText,
            ));

            // Row of 4 rune buttons (Q, W, E, R)
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(RUNE_BUTTON_GAP),
                    ..default()
                })
                .with_children(|row| {
                    for rune in [Rune::Q, Rune::W, Rune::E, Rune::R] {
                        row.spawn((
                            Button,
                            Node {
                                width: Val::Px(RUNE_BUTTON_STYLE.width),
                                height: Val::Px(RUNE_BUTTON_STYLE.height),
                                border: UiRect::all(Val::Px(RUNE_BUTTON_STYLE.border_width)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor::all(RUNE_BUTTON_STYLE.border),
                            BorderRadius::all(Val::Px(4.0)),
                            BackgroundColor(RUNE_BUTTON_STYLE.background),
                            ButtonColors {
                                background: RUNE_BUTTON_STYLE.background,
                                border: RUNE_BUTTON_STYLE.border,
                            },
                            RuneButton { rune },
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new(format!("{}", rune.as_char())),
                                TextFont {
                                    font_size: RUNE_BUTTON_STYLE.font_size,
                                    ..default()
                                },
                                TextColor(RUNE_BUTTON_STYLE.text_color),
                            ));
                        });
                    }
                });
        });
}

/// Handles rune button clicks by adding the rune to the sequence.
pub(super) fn handle_rune_button_click(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&RuneButton>,
    mut sequence: ResMut<RuneSequence>,
) {
    for event in button_clicked.read() {
        if let Ok(rune_button) = button_query.get(event.button) {
            // Add rune to sequence (respects max length and prevents duplicates)
            if sequence.len() < crate::game::runes::constants::MAX_RUNE_SEQUENCE_LENGTH {
                sequence.push(rune_button.rune);
            }
        }
    }
}

/// Updates the rune sequence display text based on current sequence.
/// Shows spell name briefly when a valid sequence is activated, then fades out.
pub(super) fn update_rune_display(
    sequence: Res<RuneSequence>,
    mut sequence_text_query: Query<
        (&mut Text, Option<&SpellNameFadeTimer>),
        With<RuneSequenceText>,
    >,
) {
    if !sequence.is_changed() {
        return;
    }

    if let Ok((mut text, fade_timer)) = sequence_text_query.single_mut() {
        // Don't update if we're in the middle of a fade animation
        if fade_timer.is_some() {
            return;
        }

        if sequence.is_empty() {
            **text = "".to_string();
        } else {
            **text = format!("{}", *sequence);
        }
    }
}

/// System to handle the spell name fade timer and remove it when done.
pub(super) fn update_spell_name_fade(
    time: Res<Time>,
    mut commands: Commands,
    mut fade_query: Query<
        (Entity, &mut SpellNameFadeTimer, &mut TextColor),
        With<RuneSequenceText>,
    >,
    mut text_query: Query<&mut Text, With<RuneSequenceText>>,
) {
    for (entity, mut timer, mut color) in &mut fade_query {
        timer.elapsed += time.delta_secs();

        // Calculate fade alpha (1.0 to 0.0)
        let alpha = (1.0 - (timer.elapsed / timer.duration)).max(0.0);
        color.0.set_alpha(alpha);

        // Remove timer and reset text when fade is complete
        if timer.elapsed >= timer.duration {
            commands.entity(entity).remove::<SpellNameFadeTimer>();
            color.0.set_alpha(1.0);

            if let Ok(mut text) = text_query.single_mut() {
                **text = "".to_string();
            }
        }
    }
}

/// Shows spell name briefly when a valid rune sequence is activated.
/// MUST run BEFORE handle_rune_activation clears the sequence.
pub(super) fn show_spell_name_on_activation(
    mut activation_reader: MessageReader<crate::game::runes::events::ActivateRuneSequence>,
    sequence: Res<RuneSequence>,
    mut commands: Commands,
    mut sequence_text_query: Query<(Entity, &mut Text), With<RuneSequenceText>>,
) {
    // Peek at messages without consuming them
    let messages: Vec<_> = activation_reader.read().collect();

    for _ in messages {
        // Check if the current sequence is valid (before it gets cleared)
        if let Some(spell) = sequence_to_spell(&sequence.runes)
            && let Ok((entity, mut text)) = sequence_text_query.single_mut()
        {
            // Show spell name
            **text = spell.name().to_string();

            // Add fade timer
            commands.entity(entity).insert(SpellNameFadeTimer {
                elapsed: 0.0,
                duration: SPELL_NAME_FADE_DURATION,
            });
        }
    }
}
