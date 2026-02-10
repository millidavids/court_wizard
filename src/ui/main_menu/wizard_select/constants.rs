//! Wizard select screen styling constants.

use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

/// Font size for the wizard select title text.
pub(super) const TITLE_FONT_SIZE: f32 = 64.0;

/// Font size for the name input display.
pub(super) const INPUT_FONT_SIZE: f32 = 32.0;

/// Text color for wizard select screen UI elements.
pub(super) const TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.9, 1.0);

/// Margin between wizard select screen UI elements in pixels.
pub(super) const MARGIN: f32 = 20.0;

/// Color for the "slots full" warning text.
pub(super) const WARNING_COLOR: Color = Color::hsla(0.0, 0.6, 0.7, 1.0);

/// Color for error messages.
pub(super) const ERROR_COLOR: Color = Color::hsla(0.0, 0.7, 0.6, 1.0);

/// Background color for the name input field.
pub(super) const INPUT_BACKGROUND: Color = Color::hsla(0.0, 0.0, 0.1, 1.0);

/// Border color for the name input field.
pub(super) const INPUT_BORDER: Color = Color::hsla(0.0, 0.0, 0.4, 1.0);

/// Maximum length for wizard names.
pub(super) const MAX_NAME_LENGTH: usize = 20;

/// Button style configuration for the wizard select screen.
pub(super) const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 250.0,
    height: 65.0,
    border_width: 3.0,
    font_size: 28.0,
    background: Color::hsla(0.0, 0.0, 0.15, 1.0),
    border: Color::hsla(0.0, 0.0, 0.3, 1.0),
    text_color: Color::hsla(0.0, 0.0, 0.9, 1.0),
};
