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

/// Border width for buttons in pixels.
pub const BUTTON_BORDER_WIDTH: f32 = 2.0;

/// Normal button background color (warm dark).
pub const BUTTON_BACKGROUND: Color = Color::hsla(25.0, 0.12, 0.13, 1.0);

/// Selected option button background color (purple highlight).
pub const SELECTED_BACKGROUND: Color = Color::hsla(270.0, 0.40, 0.25, 1.0);

/// Button border color (purple).
pub const BUTTON_BORDER: Color = Color::hsla(270.0, 0.35, 0.35, 1.0);

/// Selected option button border color (bright purple).
pub const SELECTED_BORDER: Color = Color::hsla(270.0, 0.65, 0.55, 1.0);

/// Danger button background color (red).
pub const DANGER_BUTTON_BACKGROUND: Color = Color::hsla(0.0, 0.7, 0.3, 1.0);

/// Danger button border color (red).
pub const DANGER_BUTTON_BORDER: Color = Color::hsla(0.0, 0.8, 0.5, 1.0);

/// Confirmation popup overlay background (semi-transparent black).
pub const POPUP_OVERLAY_BG: Color = OVERLAY_BG;

/// Confirmation popup box background (warm dark).
pub const POPUP_BOX_BG: Color = Color::hsla(20.0, 0.10, 0.10, 1.0);

/// Tab button height.
pub const TAB_HEIGHT: f32 = 36.0;
/// Tab horizontal padding.
pub const TAB_PADDING_H: f32 = 14.0;
/// Tab font size.
pub const TAB_FONT_SIZE: f32 = 16.0;
/// Active tab background.
pub const ACTIVE_TAB_BG: Color = Color::hsla(25.0, 0.15, 0.14, 0.8);
/// Inactive tab background.
pub const INACTIVE_TAB_BG: Color = Color::hsla(20.0, 0.10, 0.08, 0.6);
/// Active tab border.
pub const ACTIVE_TAB_BORDER: Color = Color::hsla(270.0, 0.60, 0.50, 0.8);
/// Tab border color.
pub const TAB_BORDER_COLOR: Color = Color::hsla(270.0, 0.20, 0.25, 0.6);

/// Locked/unavailable section title color.
pub const LOCKED_TITLE_COLOR: Color = Color::hsla(0.0, 0.0, 0.5, 1.0);
/// Locked/unavailable section body text color.
pub const LOCKED_TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.45, 1.0);

/// Available windowed resolution presets (16:9 only).
pub const RESOLUTION_PRESETS: &[(f32, f32, &str)] = &[
    (1280.0, 720.0, "720p"),
    (1600.0, 900.0, "900p"),
    (1920.0, 1080.0, "1080p"),
    (2560.0, 1440.0, "1440p"),
    (3840.0, 2160.0, "4K"),
];
