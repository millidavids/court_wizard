use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::runes::RuneSequence;
use crate::game::runes::constants::is_valid_sequence;

/// Spawns the rune display UI.
pub(super) fn spawn_rune_display(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(BOTTOM_MARGIN),
                right: Val::Px(RIGHT_MARGIN),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(PADDING)),
                border: UiRect::all(Val::Px(2.0)),
                min_width: Val::Px(MIN_WIDTH),
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                ..default()
            },
            BorderColor::all(BORDER_COLOR),
            BorderRadius::all(Val::Px(4.0)),
            BackgroundColor(BACKGROUND_COLOR),
            RuneDisplayRoot,
            OnGameplayScreen,
        ))
        .with_children(|parent| {
            // Rune sequence text
            parent.spawn((
                Text::new("Q W E R"),
                TextFont {
                    font_size: RUNE_SEQUENCE_FONT_SIZE,
                    ..default()
                },
                TextColor(SEQUENCE_TEXT_COLOR),
                RuneSequenceText,
            ));

            // Validity indicator text
            parent.spawn((
                Text::new("Press runes to begin"),
                TextFont {
                    font_size: VALIDITY_FONT_SIZE,
                    ..default()
                },
                TextColor(SEQUENCE_TEXT_COLOR),
                RuneValidityText,
            ));
        });
}

/// Updates the rune display UI based on current sequence.
pub(super) fn update_rune_display(
    sequence: Res<RuneSequence>,
    mut sequence_text_query: Query<&mut Text, (With<RuneSequenceText>, Without<RuneValidityText>)>,
    mut validity_query: Query<
        (&mut Text, &mut TextColor),
        (With<RuneValidityText>, Without<RuneSequenceText>),
    >,
) {
    if !sequence.is_changed() {
        return;
    }

    // Update sequence text
    if let Ok(mut text) = sequence_text_query.single_mut() {
        if sequence.is_empty() {
            **text = "Q W E R".to_string();
        } else {
            **text = format!("{}", *sequence);
        }
    }

    // Update validity indicator
    if let Ok((mut text, mut color)) = validity_query.single_mut() {
        if sequence.is_empty() {
            **text = "Press runes to begin".to_string();
            color.0 = SEQUENCE_TEXT_COLOR;
        } else if is_valid_sequence(&sequence.runes) {
            **text = "Press SPACE to cast".to_string();
            color.0 = VALID_COLOR;
        } else {
            **text = "Invalid combination".to_string();
            color.0 = INVALID_COLOR;
        }
    }
}
