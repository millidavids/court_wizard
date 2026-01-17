//! Settings menu styling constants.

use bevy::prelude::*;

/// Text color for settings menu UI elements.
pub const TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.9, 1.0);

/// Font size for settings title text.
pub const TITLE_FONT_SIZE: f32 = 48.0;

/// Font size for section headers.
pub const SECTION_FONT_SIZE: f32 = 28.0;

/// Font size for option labels and values.
pub const LABEL_FONT_SIZE: f32 = 20.0;

/// Font size for button text.
pub const BUTTON_FONT_SIZE: f32 = 18.0;

/// Margin between settings UI elements in pixels.
pub const MARGIN: f32 = 20.0;

/// Small margin for tighter spacing.
pub const MARGIN_SMALL: f32 = 10.0;

/// Width of option buttons in pixels.
pub const OPTION_BUTTON_WIDTH: f32 = 120.0;

/// Height of option buttons in pixels.
pub const OPTION_BUTTON_HEIGHT: f32 = 40.0;

/// Width of volume control buttons in pixels.
pub const VOLUME_BUTTON_SIZE: f32 = 30.0;

/// Width of the Back button in pixels.
pub const BACK_BUTTON_WIDTH: f32 = 150.0;

/// Height of the Back button in pixels.
pub const BACK_BUTTON_HEIGHT: f32 = 50.0;

/// Border width for buttons in pixels.
pub const BUTTON_BORDER_WIDTH: f32 = 2.0;

/// Normal button background color.
pub const BUTTON_BACKGROUND: Color = Color::hsla(0.0, 0.0, 0.15, 1.0);

/// Selected option button background color.
pub const SELECTED_BACKGROUND: Color = Color::hsla(210.0, 0.7, 0.4, 1.0);

/// Button border color.
pub const BUTTON_BORDER: Color = Color::hsla(0.0, 0.0, 0.4, 1.0);

/// Selected option button border color.
pub const SELECTED_BORDER: Color = Color::hsla(210.0, 0.8, 0.6, 1.0);
