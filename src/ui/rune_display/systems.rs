use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::game_mode::components::ArchetypeUI;
use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::game::input::messages::MouseClicked;
use crate::game::units::wizard::archetypes::runes::resources::Rune;
use crate::game::units::wizard::archetypes::runes::{LastActivatedSpell, RuneSequence};
use crate::ui::button_systems::{edge_color, opaque};
use crate::ui::components::{ButtonAnimState, ButtonColors, ButtonEdge, ButtonFront};
use crate::ui::constants::{BUTTON_3D_OFFSET_REST, BUTTON_REST_OUTLINE, BUTTON_SHADOW_COLOR};
use crate::ui::focus::Focusable;
use crate::ui::gamepad_glyphs::{CurrentControllerGlyphStyle, GamepadGlyphFonts, glyph_char};

/// Spawns the rune display UI with 4 clickable buttons and sequence text above.
pub(crate) fn spawn_rune_display(mut commands: Commands) {
    // Create a full-width container at the bottom for proper centering
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(BOTTOM_MARGIN),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            OnGameplayScreen,
            ArchetypeUI,
        ))
        .with_children(|parent| {
            // Inner container with the actual rune buttons
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                    RuneDisplayRoot,
                ))
                .with_children(|inner| {
                    // Activated spell name (above sequence text, fades out)
                    let activated_font = RUNE_SEQUENCE_FONT_SIZE + 4.0;
                    let activated_offset = activated_font / 20.0;
                    inner
                        .spawn(Node {
                            position_type: PositionType::Relative,
                            min_height: Val::Px(activated_font + 8.0),
                            ..default()
                        })
                        .with_children(|wrapper| {
                            // Shadow
                            wrapper.spawn((
                                Text::new(""),
                                TextFont::from_font_size(activated_font),
                                TextColor(Color::NONE),
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(activated_offset),
                                    top: Val::Px(activated_offset),
                                    ..default()
                                },
                                ActivatedSpellTextShadow,
                            ));
                            // Main text
                            wrapper.spawn((
                                Text::new(""),
                                TextFont::from_font_size(activated_font),
                                TextColor(Color::srgba(0.7, 0.5, 1.0, 0.0)),
                                ActivatedSpellText,
                            ));
                        });

                    // Sequence text above buttons
                    let seq_offset = RUNE_SEQUENCE_FONT_SIZE / 20.0;
                    inner
                        .spawn(Node {
                            position_type: PositionType::Relative,
                            min_height: Val::Px(RUNE_SEQUENCE_FONT_SIZE + 4.0),
                            ..default()
                        })
                        .with_children(|wrapper| {
                            // Shadow
                            wrapper.spawn((
                                Text::new(""),
                                TextFont::from_font_size(RUNE_SEQUENCE_FONT_SIZE),
                                TextColor(crate::ui::constants::TEXT_SHADOW_COLOR),
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(seq_offset),
                                    top: Val::Px(seq_offset),
                                    ..default()
                                },
                                RuneSequenceTextShadow,
                            ));
                            // Main text
                            wrapper.spawn((
                                Text::new(""),
                                TextFont::from_font_size(RUNE_SEQUENCE_FONT_SIZE),
                                TextColor(SEQUENCE_TEXT_COLOR),
                                RuneSequenceText,
                            ));
                        });

                    // Row of 4 rune buttons (Q, W, E, R)
                    inner
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(RUNE_BUTTON_GAP),
                            ..default()
                        })
                        .with_children(|row| {
                            for rune in [Rune::Q, Rune::W, Rune::E, Rune::R] {
                                spawn_rune_button(row, rune);
                            }
                        });
                });
        });
}

/// Spawns one rune button with the shared 3D press/hover/release structure
/// (edge + front layers + `ButtonAnimState`, matching `layout_helpers::spawn_button`).
/// Built inline rather than via `spawn_button` because the front text needs a
/// per-rune `RuneButtonLabel` marker that `adapt_rune_labels_to_input_device`
/// updates for gamepad glyphs — something `spawn_button` can't tag.
fn spawn_rune_button(row: &mut ChildSpawnerCommands, rune: Rune) {
    let depth = -BUTTON_3D_OFFSET_REST; // positive: edge visible at the bottom
    row.spawn((
        Button,
        Node {
            width: Val::Px(RUNE_BUTTON_STYLE.width),
            height: Val::Px(RUNE_BUTTON_STYLE.height + depth),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            position_type: PositionType::Relative,
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(Color::NONE),
        BoxShadow(vec![ShadowStyle {
            color: BUTTON_SHADOW_COLOR,
            x_offset: Val::Px(0.0),
            y_offset: Val::Px(2.0),
            spread_radius: Val::Px(0.0),
            blur_radius: Val::Px(4.0),
        }]),
        ButtonColors {
            background: RUNE_BUTTON_STYLE.background,
            border: RUNE_BUTTON_STYLE.border,
        },
        ButtonAnimState {
            current: BUTTON_3D_OFFSET_REST,
            target: BUTTON_3D_OFFSET_REST,
        },
        Focusable,
        RuneButton { rune },
    ))
    .with_children(|wrapper| {
        // Edge layer (sits behind the front face).
        wrapper.spawn((
            ButtonEdge,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(edge_color(RUNE_BUTTON_STYLE.background)),
            Outline::new(Val::Px(1.0), Val::Px(1.0), BUTTON_REST_OUTLINE),
        ));
        // Front face — moves on press/hover; holds the rune label.
        wrapper
            .spawn((
                ButtonFront,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(RUNE_BUTTON_STYLE.height),
                    border: UiRect::all(Val::Px(RUNE_BUTTON_STYLE.border_width)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Relative,
                    top: Val::Px(BUTTON_3D_OFFSET_REST),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(opaque(RUNE_BUTTON_STYLE.background)),
                BorderColor::all(RUNE_BUTTON_STYLE.border),
            ))
            .with_children(|front| {
                front.spawn((
                    Text::new(format!("{}", rune.as_char())),
                    TextFont::from_font_size(RUNE_BUTTON_STYLE.font_size),
                    TextColor(RUNE_BUTTON_STYLE.text_color),
                    RuneButtonLabel { rune },
                ));
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
            if sequence.len()
                < crate::game::units::wizard::archetypes::runes::constants::MAX_RUNE_SEQUENCE_LENGTH
            {
                sequence.push(rune_button.rune);
            }
        }
    }
}

/// Updates the rune sequence display text based on current sequence.
/// Shows spell name briefly when a valid sequence is activated, then fades out.
pub(super) fn update_rune_display(
    sequence: Res<RuneSequence>,
    mut commands: Commands,
    mut sequence_text_query: Query<
        (
            Entity,
            &mut Text,
            Option<&SpellNameFadeTimer>,
            &mut TextColor,
        ),
        With<RuneSequenceText>,
    >,
    mut shadow_query: Query<&mut Text, (With<RuneSequenceTextShadow>, Without<RuneSequenceText>)>,
) {
    if !sequence.is_changed() {
        return;
    }

    if let Ok((entity, mut text, fade_timer, mut color)) = sequence_text_query.single_mut() {
        if !sequence.is_empty() && fade_timer.is_some() {
            commands.entity(entity).remove::<SpellNameFadeTimer>();
            color.0.set_alpha(1.0);
        }

        if fade_timer.is_some() && sequence.is_empty() {
            return;
        }

        let new_text = if sequence.is_empty() {
            "".to_string()
        } else {
            format!("{}", *sequence)
        };
        **text = new_text.clone();
        if let Ok(mut shadow) = shadow_query.single_mut() {
            **shadow = new_text;
        }
    }
}

/// System to handle the activated spell name fade timer.
pub(super) fn update_spell_name_fade(
    time: Res<Time>,
    mut commands: Commands,
    mut fade_query: Query<
        (Entity, &mut SpellNameFadeTimer, &mut TextColor),
        With<ActivatedSpellText>,
    >,
    mut shadow_query: Query<
        (&mut Text, &mut TextColor),
        (With<ActivatedSpellTextShadow>, Without<ActivatedSpellText>),
    >,
) {
    for (entity, mut timer, mut color) in &mut fade_query {
        timer.elapsed += time.delta_secs();

        let alpha = (1.0 - (timer.elapsed / timer.duration)).max(0.0);
        color.0.set_alpha(alpha);

        // Fade shadow in sync
        if let Ok((_, mut shadow_color)) = shadow_query.single_mut() {
            shadow_color.0.set_alpha(alpha * 0.5);
        }

        if timer.elapsed >= timer.duration {
            commands.entity(entity).remove::<SpellNameFadeTimer>();
            color.0.set_alpha(0.0);

            // Clear both texts
            if let Ok((mut shadow_text, mut shadow_color)) = shadow_query.single_mut() {
                **shadow_text = "".to_string();
                shadow_color.0.set_alpha(0.0);
            }
        }
    }
}

/// Shows spell name briefly above the rune display when a valid rune sequence is activated.
pub(super) fn show_spell_name_on_activation(
    mut last_activated: ResMut<LastActivatedSpell>,
    mut commands: Commands,
    mut activated_text_query: Query<(Entity, &mut Text, &mut TextColor), With<ActivatedSpellText>>,
    mut shadow_query: Query<
        (&mut Text, &mut TextColor),
        (With<ActivatedSpellTextShadow>, Without<ActivatedSpellText>),
    >,
) {
    if last_activated.just_activated {
        if let Some(spell) = last_activated.spell
            && let Ok((entity, mut text, mut text_color)) = activated_text_query.single_mut()
        {
            let name = spell.name().to_string();
            **text = name.clone();
            text_color.0 = Color::srgba(0.7, 0.5, 1.0, 1.0);

            // Sync shadow
            if let Ok((mut shadow_text, mut shadow_color)) = shadow_query.single_mut() {
                **shadow_text = name;
                shadow_color.0 = crate::ui::constants::TEXT_SHADOW_COLOR;
            }

            commands.entity(entity).insert(SpellNameFadeTimer {
                elapsed: 0.0,
                duration: SPELL_NAME_FADE_DURATION,
            });
        }
        last_activated.acknowledge();
    }
}

/// Swaps each rune button's "Q/W/E/R" label for a D-pad direction glyph
/// when a gamepad is the active input device. Mapping matches the gameplay
/// binding laid out in the controller plan: Q=Up, W=Down, E=Right, R=Left.
pub(super) fn adapt_rune_labels_to_input_device(
    active: Res<ActiveInputDevice>,
    style: Res<CurrentControllerGlyphStyle>,
    fonts: Option<Res<GamepadGlyphFonts>>,
    mut labels: Query<(&RuneButtonLabel, &mut Text, &mut TextFont)>,
) {
    let gamepad = active.is_gamepad();
    for (label, mut text, mut font) in &mut labels {
        if gamepad && let Some(ref fonts) = fonts {
            let button = match label.rune {
                Rune::Q => GamepadButton::DPadUp,
                Rune::W => GamepadButton::DPadDown,
                Rune::E => GamepadButton::DPadRight,
                Rune::R => GamepadButton::DPadLeft,
            };
            if let Some(glyph) = glyph_char(button, style.0) {
                let s = glyph.to_string();
                if **text != s {
                    **text = s;
                }
                let want = fonts.font_for(style.0);
                if font.font != want {
                    font.font = want;
                    font.font_size = RUNE_BUTTON_STYLE.font_size * 1.4;
                }
            }
        } else {
            let s = format!("{}", label.rune.as_char());
            if **text != s {
                **text = s;
            }
            if font.font != Handle::<Font>::default() {
                font.font = Handle::<Font>::default();
                font.font_size = RUNE_BUTTON_STYLE.font_size;
            }
        }
    }
}
