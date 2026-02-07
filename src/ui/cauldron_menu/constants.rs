use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

pub const BACKGROUND_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.8);
pub const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
pub const TITLE_FONT_SIZE: f32 = 60.0;
pub const DESCRIPTION_FONT_SIZE: f32 = 16.0;
pub const BREW_TIME_FONT_SIZE: f32 = 14.0;
pub const BREW_TIME_COLOR: Color = Color::srgb(0.7, 0.7, 0.5);
pub const BUTTON_FONT_SIZE: f32 = 24.0;
pub const BUTTON_WIDTH: f32 = 220.0;
pub const BUTTON_HEIGHT: f32 = 60.0;
pub const BUTTON_BORDER_WIDTH: f32 = 2.0;
pub const BUTTON_BACKGROUND: Color = Color::srgb(0.15, 0.15, 0.15);
pub const BUTTON_BORDER: Color = Color::srgb(0.4, 0.4, 0.4);
pub const MARGIN: f32 = 20.0;
pub const BREW_COLUMN_WIDTH: f32 = 220.0;
pub const BREW_COLUMN_GAP: f32 = 16.0;
pub const COLUMN_PADDING: f32 = 20.0;
pub const FRAME_BORDER_WIDTH: f32 = 2.0;
pub const FRAME_BORDER_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);
pub const FRAME_BACKGROUND: Color = Color::srgba(0.1, 0.1, 0.1, 0.6);
pub const FRAME_PADDING: f32 = 12.0;

/// Button style configuration for the cauldron menu.
pub const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: BUTTON_WIDTH,
    height: BUTTON_HEIGHT,
    border_width: BUTTON_BORDER_WIDTH,
    font_size: BUTTON_FONT_SIZE,
    background: BUTTON_BACKGROUND,
    border: BUTTON_BORDER,
    text_color: TEXT_COLOR,
};

/// Button style for disabled/grayed-out brew buttons during brewing.
pub const DISABLED_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: BUTTON_WIDTH,
    height: BUTTON_HEIGHT,
    border_width: BUTTON_BORDER_WIDTH,
    font_size: BUTTON_FONT_SIZE,
    background: Color::srgb(0.1, 0.1, 0.1),
    border: Color::srgb(0.25, 0.25, 0.25),
    text_color: Color::srgb(0.4, 0.4, 0.4),
};

/// Button style for the cancel brew button.
pub const CANCEL_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 300.0,
    height: 60.0,
    border_width: BUTTON_BORDER_WIDTH,
    font_size: BUTTON_FONT_SIZE,
    background: Color::srgb(0.3, 0.1, 0.1),
    border: Color::srgb(0.6, 0.2, 0.2),
    text_color: Color::srgb(0.9, 0.4, 0.4),
};

pub const BREWING_STATUS_FONT_SIZE: f32 = 20.0;
pub const BREWING_STATUS_COLOR: Color = Color::srgb(0.9, 0.7, 0.3);
pub const DISABLED_TEXT_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);
pub const DISABLED_BREW_TIME_COLOR: Color = Color::srgb(0.35, 0.35, 0.25);

/// Button style for the close button (wider).
pub const CLOSE_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 300.0,
    height: 70.0,
    border_width: BUTTON_BORDER_WIDTH,
    font_size: 32.0,
    background: BUTTON_BACKGROUND,
    border: BUTTON_BORDER,
    text_color: TEXT_COLOR,
};
