//! Settings menu styling constants.

use bevy::prelude::*;

use crate::ui::constants::{OVERLAY_BG, TEXT_PRIMARY};

/// Text color for settings menu UI elements.
pub const TEXT_COLOR: Color = TEXT_PRIMARY;

/// Font size for settings title text.
pub const TITLE_FONT_SIZE: f32 = 32.0;

/// Font size for section headers.
pub const SECTION_FONT_SIZE: f32 = 18.0;

/// Font size for option labels and values.
pub const LABEL_FONT_SIZE: f32 = 14.0;

/// Font size for button text.
pub const BUTTON_FONT_SIZE: f32 = 13.0;

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

/// Danger button background color (red).
pub const DANGER_BUTTON_BACKGROUND: Color = Color::hsla(0.0, 0.7, 0.3, 1.0);

/// Danger button border color (red).
pub const DANGER_BUTTON_BORDER: Color = Color::hsla(0.0, 0.8, 0.5, 1.0);

/// Confirmation popup overlay background (semi-transparent black).
pub const POPUP_OVERLAY_BG: Color = OVERLAY_BG;

/// Confirmation popup box background.
pub const POPUP_BOX_BG: Color = Color::hsla(220.0, 0.08, 0.12, 1.0);
