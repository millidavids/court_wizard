//! Save select screen styling constants.

use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

/// Font size for the save select title text.
pub(super) const TITLE_FONT_SIZE: f32 = 64.0;

/// Text color for save select screen UI elements.
pub(super) const TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.9, 1.0);

/// Margin between save select screen UI elements in pixels.
pub(super) const MARGIN: f32 = 20.0;

/// Button style configuration for the save select screen.
pub(super) const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 300.0,
    height: 65.0,
    border_width: 3.0,
    font_size: 24.0,
    background: Color::hsla(0.0, 0.0, 0.15, 1.0),
    border: Color::hsla(0.0, 0.0, 0.3, 1.0),
    text_color: Color::hsla(0.0, 0.0, 0.9, 1.0),
};

/// Button style for delete buttons (smaller, red-tinted).
pub(super) const DELETE_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 100.0,
    height: 65.0,
    border_width: 3.0,
    font_size: 20.0,
    background: Color::hsla(0.0, 0.3, 0.15, 1.0),
    border: Color::hsla(0.0, 0.4, 0.3, 1.0),
    text_color: Color::hsla(0.0, 0.6, 0.7, 1.0),
};
