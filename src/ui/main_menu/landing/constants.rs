//! Landing screen styling constants.

use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

/// Background color for landing screen buttons.
pub const BUTTON_BACKGROUND: Color = Color::hsla(0.0, 0.0, 0.15, 1.0);

/// Border color for landing screen buttons.
pub const BUTTON_BORDER: Color = Color::hsla(0.0, 0.0, 0.3, 1.0);

/// Width for landing screen buttons in pixels.
pub const BUTTON_WIDTH: f32 = 250.0;

/// Height for landing screen buttons in pixels.
pub const BUTTON_HEIGHT: f32 = 65.0;

/// Border width for landing screen buttons in pixels.
pub const BUTTON_BORDER_WIDTH: f32 = 3.0;

/// Font size for landing screen button text.
pub const BUTTON_FONT_SIZE: f32 = 28.0;

/// Font size for landing screen title text.
pub const TITLE_FONT_SIZE: f32 = 64.0;

/// Text color for landing screen UI elements.
pub const TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.9, 1.0);

/// Margin between landing screen UI elements in pixels.
pub const MARGIN: f32 = 20.0;

/// Button style configuration for the landing screen.
pub const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: BUTTON_WIDTH,
    height: BUTTON_HEIGHT,
    border_width: BUTTON_BORDER_WIDTH,
    font_size: BUTTON_FONT_SIZE,
    background: BUTTON_BACKGROUND,
    border: BUTTON_BORDER,
    text_color: TEXT_COLOR,
};
