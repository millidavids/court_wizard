//! Multiplayer screen styling constants.
//!
//! Most wizard select styling is shared with the single-player screen via `wizard_select::shared`.
//! This file only contains multiplayer-specific constants.

use bevy::prelude::*;

use crate::ui::components::ButtonStyle;
use crate::ui::constants::{
    BUTTON_BG, BUTTON_BG_SUBTLE, BUTTON_BORDER, BUTTON_BORDER_SUBTLE, ERROR_COLOR as GLOBAL_ERROR,
    SUCCESS_COLOR as GLOBAL_SUCCESS, TEXT_MUTED, WARNING_COLOR as GLOBAL_WARNING,
};

// Re-export all shared constants for convenience.
pub(super) use super::super::wizard_select::shared::*;

// ===== Connection Phase Styling =====

/// Width for multiplayer screen buttons in pixels.
const BUTTON_WIDTH: f32 = 250.0;

/// Height for multiplayer screen buttons in pixels.
const BUTTON_HEIGHT: f32 = 65.0;

/// Border width for multiplayer screen buttons in pixels.
const BUTTON_BORDER_WIDTH: f32 = 3.0;

/// Font size for multiplayer screen button text.
const BUTTON_FONT_SIZE: f32 = 20.0;

/// Font size for multiplayer screen title text.
pub(super) const MP_TITLE_FONT_SIZE: f32 = 48.0;

/// Font size for status text.
pub(super) const STATUS_FONT_SIZE: f32 = 18.0;

/// Font size for the code display text.
pub(super) const CODE_FONT_SIZE: f32 = 12.0;

/// Accent color for success/connected status.
pub(super) const SUCCESS_COLOR: Color = GLOBAL_SUCCESS;

/// Accent color for error/failed status.
pub(super) const ERROR_COLOR: Color = GLOBAL_ERROR;

/// Accent color for waiting/connecting status.
pub(super) const WAITING_COLOR: Color = GLOBAL_WARNING;

/// Color for section divider labels ("Online" / "Local Network").
pub(super) const SECTION_LABEL_COLOR: Color = TEXT_MUTED;

/// Font size for section divider labels.
pub(super) const SECTION_LABEL_FONT_SIZE: f32 = 14.0;

/// Font size for the IP address display in the LAN IP entry screen.
pub(super) const IP_DISPLAY_FONT_SIZE: f32 = 22.0;

/// Width of the left column in the connection screen layout.
pub(super) const CONN_LEFT_COLUMN_WIDTH: f32 = 280.0;

/// Button style configuration for the multiplayer connection screen.
pub(super) const CONN_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: BUTTON_WIDTH,
    height: BUTTON_HEIGHT,
    border_width: BUTTON_BORDER_WIDTH,
    font_size: BUTTON_FONT_SIZE,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: TEXT_COLOR,
    text_shadow: true,
};

// ===== Multiplayer-specific Wizard Select Styling =====

/// Button style for the Ready button in the detail panel.
pub(super) const READY_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 140.0,
    height: 36.0,
    border_width: 1.0,
    font_size: 15.0,
    background: Color::hsla(120.0, 0.30, 0.20, 0.75),
    border: Color::hsla(120.0, 0.50, 0.35, 1.0),
    text_color: Color::hsla(120.0, 0.20, 0.85, 1.0),
    text_shadow: true,
};

/// Button style for the Unready button in the detail panel.
pub(super) const UNREADY_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 140.0,
    height: 36.0,
    border_width: 1.0,
    font_size: 15.0,
    background: Color::hsla(0.0, 0.30, 0.20, 0.75),
    border: Color::hsla(0.0, 0.50, 0.35, 1.0),
    text_color: Color::hsla(0.0, 0.20, 0.85, 1.0),
    text_shadow: true,
};

/// Button style for the Disconnect button in wizard select — minimal, unobtrusive.
pub(super) const DISCONNECT_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 100.0,
    height: 34.0,
    border_width: 1.0,
    font_size: 13.0,
    background: BUTTON_BG_SUBTLE,
    border: BUTTON_BORDER_SUBTLE,
    text_color: Color::hsla(0.0, 0.0, 0.70, 1.0),
    text_shadow: true,
};
