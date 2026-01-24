//! Pause menu main screen styling constants.

use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

/// Text color for the title and buttons.
pub const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

/// Font size for the title text.
pub const TITLE_FONT_SIZE: f32 = 60.0;

/// Font size for button text.
pub const BUTTON_FONT_SIZE: f32 = 32.0;

/// Width of all buttons.
pub const BUTTON_WIDTH: f32 = 300.0;

/// Height of all buttons.
pub const BUTTON_HEIGHT: f32 = 70.0;

/// Button border width.
pub const BUTTON_BORDER_WIDTH: f32 = 2.0;

/// Button background color.
pub const BUTTON_BACKGROUND: Color = Color::srgb(0.15, 0.15, 0.15);

/// Button border color.
pub const BUTTON_BORDER: Color = Color::srgb(0.4, 0.4, 0.4);

/// Spacing between UI elements.
pub const MARGIN: f32 = 20.0;

/// Button style configuration for the pause menu.
pub const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: BUTTON_WIDTH,
    height: BUTTON_HEIGHT,
    border_width: BUTTON_BORDER_WIDTH,
    font_size: BUTTON_FONT_SIZE,
    background: BUTTON_BACKGROUND,
    border: BUTTON_BORDER,
    text_color: TEXT_COLOR,
};
