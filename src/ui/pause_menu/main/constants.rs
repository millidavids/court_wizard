//! Pause menu main screen styling constants.

use bevy::prelude::*;

use crate::ui::components::ButtonStyle;
use crate::ui::constants::{BUTTON_BG, BUTTON_BORDER, TEXT_MUTED, TEXT_PRIMARY};

/// Text color (alias for global TEXT_PRIMARY).
pub const TEXT_COLOR: Color = TEXT_PRIMARY;

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
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};

// ── Left Panel Stats ────────────────────────────────────────────────────────

/// Font size for stat labels (e.g., "Level", "Kills").
pub const STAT_LABEL_FONT_SIZE: f32 = 11.0;

/// Color for stat labels.
pub const STAT_LABEL_COLOR: Color = TEXT_MUTED;

/// Font size for stat values.
pub const STAT_VALUE_FONT_SIZE: f32 = 12.0;

/// Color for stat values.
pub const STAT_VALUE_COLOR: Color = Color::hsla(40.0, 0.15, 0.75, 1.0);

/// Font size for section dividers (e.g., "Modifiers", "Level Best").
pub const SECTION_DIVIDER_FONT_SIZE: f32 = 10.0;

/// Color for section dividers.
pub const SECTION_DIVIDER_COLOR: Color = TEXT_MUTED;
