//! Wizard select screen styling constants.

use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

/// Font size for the wizard select title text.
pub(super) const TITLE_FONT_SIZE: f32 = 42.0;

/// Text color for wizard select screen UI elements.
pub(super) const TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.9, 1.0);

/// Margin between wizard select screen UI elements in pixels.
pub(super) const MARGIN: f32 = 20.0;

/// Color for wizard type description text.
pub(super) const DESCRIPTION_COLOR: Color = Color::hsla(0.0, 0.0, 0.6, 1.0);

/// Color for stat text (e.g., "Highest Level: 5").
pub(super) const STAT_COLOR: Color = Color::hsla(45.0, 0.5, 0.6, 1.0);

/// Color for "New" indicator text.
pub(super) const NEW_COLOR: Color = Color::hsla(120.0, 0.4, 0.5, 1.0);

/// Button style configuration for the wizard select screen.
pub(super) const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 250.0,
    height: 65.0,
    border_width: 3.0,
    font_size: 18.0,
    background: Color::hsla(0.0, 0.0, 0.15, 1.0),
    border: Color::hsla(0.0, 0.0, 0.3, 1.0),
    text_color: Color::hsla(0.0, 0.0, 0.9, 1.0),
};
