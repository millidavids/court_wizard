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
pub const BUTTON_FONT_SIZE: f32 = 20.0;

/// Font size for landing screen title text.
pub const TITLE_FONT_SIZE: f32 = 96.0;

/// Text color for landing screen UI elements.
pub const TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.9, 1.0);

/// Margin between landing screen UI elements in pixels.
pub const MARGIN: f32 = 20.0;

/// Left padding for the button column on the landing screen.
pub const BUTTONS_LEFT_PADDING: f32 = 80.0;

/// Title text drop shadow color.
pub const TITLE_SHADOW_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.5);

/// Title text drop shadow offset in pixels (x and y).
pub const TITLE_SHADOW_OFFSET: f32 = 4.0;

/// Back button style shared across sub-screens (changelog, credits, etc.).
pub const BACK_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 150.0,
    height: 50.0,
    border_width: 2.0,
    font_size: 18.0,
    background: BUTTON_BACKGROUND,
    border: BUTTON_BORDER,
    text_color: TEXT_COLOR,
};

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
