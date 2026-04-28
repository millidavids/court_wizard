//! Controller binding diagram rendered at the bottom of the Controller settings tab.
//!
//! Tracks `CurrentControllerGlyphStyle` (resolved to the connected gamepad's
//! vendor, or Xbox if none) and renders a labeled grid of bindings using the
//! existing Kenney input fonts.

use bevy::input::gamepad::GamepadButton;
use bevy::prelude::*;

use crate::config::ControllerGlyphStyle;
use crate::ui::constants::{CONTENT_BG, DETAIL_BORDER, PANEL_BORDER_RADIUS, WARNING_COLOR};
use crate::ui::gamepad_glyphs::{GamepadGlyphFonts, glyph_char};

use super::constants::{
    BUTTON_BORDER_WIDTH, LABEL_FONT_SIZE, MARGIN_SMALL, SECTION_FONT_SIZE, TEXT_COLOR,
};

const GLYPH_SIZE: f32 = 56.0;
/// Tight vertical spacing reused across the panel's stacked sections.
const ROW_GAP: f32 = 4.0;

struct Binding {
    buttons: &'static [GamepadButton],
    label: &'static str,
}

const LEFT_BINDINGS: &[Binding] = &[
    Binding {
        buttons: &[GamepadButton::LeftThumb],
        label: "Aim",
    },
    Binding {
        buttons: &[GamepadButton::RightTrigger2],
        label: "Cast Spell",
    },
    Binding {
        buttons: &[GamepadButton::LeftTrigger2],
        label: "Cancel Cast",
    },
    Binding {
        buttons: &[GamepadButton::RightThumb],
        label: "Pick Spell",
    },
    Binding {
        buttons: &[GamepadButton::South],
        label: "Confirm",
    },
];

const RIGHT_BINDINGS: &[Binding] = &[
    Binding {
        buttons: &[GamepadButton::East],
        label: "Back / Close",
    },
    Binding {
        buttons: &[GamepadButton::West],
        label: "Spell Book",
    },
    Binding {
        buttons: &[GamepadButton::North],
        label: "Cauldron",
    },
    Binding {
        buttons: &[GamepadButton::LeftTrigger, GamepadButton::RightTrigger],
        label: "Cycle Tabs",
    },
    Binding {
        buttons: &[GamepadButton::Start],
        label: "Pause",
    },
];

/// Sized to fit two glyphs side-by-side (e.g. LB+RB) so single-glyph rows
/// align with two-glyph rows.
const GLYPH_CELL_WIDTH: f32 = GLYPH_SIZE * 2.0 + 8.0;

fn title_for(style: ControllerGlyphStyle) -> &'static str {
    match style {
        ControllerGlyphStyle::PlayStation => "PLAYSTATION",
        ControllerGlyphStyle::SteamDeck => "STEAM",
        ControllerGlyphStyle::Switch => "SWITCH",
        ControllerGlyphStyle::Auto | ControllerGlyphStyle::Xbox => "XBOX",
    }
}

pub(super) fn spawn_controller_diagram_section(
    parent: &mut ChildSpawnerCommands,
    glyph_fonts: &GamepadGlyphFonts,
    style: ControllerGlyphStyle,
) {
    let glyph_font = glyph_fonts.font_for(style);

    parent
        .spawn((
            Node {
                width: Val::Percent(97.0),
                margin: UiRect {
                    top: Val::Px(MARGIN_SMALL),
                    bottom: Val::Px(MARGIN_SMALL),
                    ..default()
                },
                padding: UiRect::all(Val::Px(6.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(ROW_GAP),
                border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                border_radius: BorderRadius::all(Val::Px(PANEL_BORDER_RADIUS)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(CONTENT_BG),
            BorderColor::all(DETAIL_BORDER),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(title_for(style)),
                TextFont::from_font_size(SECTION_FONT_SIZE),
                TextColor(WARNING_COLOR),
            ));

            panel.spawn((
                Node {
                    width: Val::Percent(60.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(ROW_GAP)),
                    ..default()
                },
                BackgroundColor(DETAIL_BORDER),
            ));

            panel
                .spawn(Node {
                    min_width: Val::Px(0.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(32.0),
                    margin: UiRect::vertical(Val::Px(ROW_GAP)),
                    ..default()
                })
                .with_children(|cols| {
                    spawn_binding_column(cols, &glyph_font, style, LEFT_BINDINGS);
                    spawn_binding_column(cols, &glyph_font, style, RIGHT_BINDINGS);
                });

            panel.spawn((
                Text::new("PICK SPELL = HOLD RIGHT STICK + CAST"),
                TextFont::from_font_size(10.0),
                TextColor(DETAIL_BORDER),
                Node {
                    margin: UiRect::top(Val::Px(ROW_GAP)),
                    ..default()
                },
            ));
        });
}

fn spawn_binding_column(
    parent: &mut ChildSpawnerCommands,
    glyph_font: &Handle<Font>,
    style: ControllerGlyphStyle,
    bindings: &[Binding],
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(ROW_GAP),
            min_width: Val::Px(0.0),
            ..default()
        })
        .with_children(|column| {
            for binding in bindings {
                spawn_binding_row(column, glyph_font, style, binding);
            }
        });
}

fn spawn_binding_row(
    parent: &mut ChildSpawnerCommands,
    glyph_font: &Handle<Font>,
    style: ControllerGlyphStyle,
    binding: &Binding,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(MARGIN_SMALL),
            ..default()
        })
        .with_children(|row| {
            row.spawn(Node {
                width: Val::Px(GLYPH_CELL_WIDTH),
                height: Val::Px(GLYPH_SIZE + 4.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|cell| {
                for &button in binding.buttons {
                    if let Some(ch) = glyph_char(button, style) {
                        cell.spawn((
                            Text::new(ch.to_string()),
                            TextFont {
                                font: glyph_font.clone(),
                                font_size: GLYPH_SIZE,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        ));
                    }
                }
            });

            row.spawn((
                Text::new(binding.label),
                TextFont::from_font_size(LABEL_FONT_SIZE),
                TextColor(TEXT_COLOR),
            ));
        });
}
