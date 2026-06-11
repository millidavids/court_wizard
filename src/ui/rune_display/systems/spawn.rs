use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::game_mode::components::ArchetypeUI;
use crate::game::units::wizard::archetypes::runes::resources::Rune;
use crate::ui::button_systems::{edge_color, opaque};
use crate::ui::components::{ButtonAnimState, ButtonColors, ButtonEdge, ButtonFront};
use crate::ui::constants::{BUTTON_3D_OFFSET_REST, BUTTON_SHADOW_COLOR};
use crate::ui::focus::Focusable;

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
    use crate::ui::constants::BUTTON_REST_OUTLINE;
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
