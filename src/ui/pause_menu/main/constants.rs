//! Pause menu main screen styling constants.

use bevy::prelude::*;

use crate::ui::components::{ButtonStyle, BUTTON_BG, BUTTON_BORDER};

/// Text color for the title and buttons.
pub const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

/// Font size for the title text.
pub const TITLE_FONT_SIZE: f32 = 40.0;

/// Font size for button text.
pub const BUTTON_FONT_SIZE: f32 = 22.0;

/// Width of all buttons.
pub const BUTTON_WIDTH: f32 = 300.0;

/// Height of all buttons.
pub const BUTTON_HEIGHT: f32 = 70.0;

/// Button border width.
pub const BUTTON_BORDER_WIDTH: f32 = 2.0;

/// Spacing between UI elements.
pub const MARGIN: f32 = 20.0;

/// Button style configuration for the pause menu.
pub const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: BUTTON_WIDTH,
    height: BUTTON_HEIGHT,
    border_width: BUTTON_BORDER_WIDTH,
    font_size: BUTTON_FONT_SIZE,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: TEXT_COLOR,
    text_shadow: true,
};
