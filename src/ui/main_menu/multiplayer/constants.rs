//! Multiplayer screen styling constants.
//!
//! Most wizard select styling is shared with the single-player screen via `wizard_select_shared`.
//! This file only contains multiplayer-specific constants.

use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

// Re-export all shared constants for convenience.
pub(super) use super::super::wizard_select_shared::*;

// ===== Connection Phase Styling =====

/// Background color for multiplayer screen buttons.
const BUTTON_BACKGROUND: Color = Color::hsla(0.0, 0.0, 0.15, 1.0);

/// Border color for multiplayer screen buttons.
const BUTTON_BORDER: Color = Color::hsla(0.0, 0.0, 0.3, 1.0);

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
pub(super) const SUCCESS_COLOR: Color = Color::hsla(120.0, 0.6, 0.5, 1.0);

/// Accent color for error/failed status.
pub(super) const ERROR_COLOR: Color = Color::hsla(0.0, 0.6, 0.5, 1.0);

/// Accent color for waiting/connecting status.
pub(super) const WAITING_COLOR: Color = Color::hsla(45.0, 0.6, 0.5, 1.0);

/// Button style configuration for the multiplayer connection screen.
pub(super) const CONN_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: BUTTON_WIDTH,
    height: BUTTON_HEIGHT,
    border_width: BUTTON_BORDER_WIDTH,
    font_size: BUTTON_FONT_SIZE,
    background: BUTTON_BACKGROUND,
    border: BUTTON_BORDER,
    text_color: TEXT_COLOR,
};

// ===== Multiplayer-specific Wizard Select Styling =====

/// Button style for the Ready button in the detail panel.
pub(super) const READY_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 140.0,
    height: 36.0,
    border_width: 1.0,
    font_size: 15.0,
    background: Color::hsla(120.0, 0.30, 0.20, 1.0),
    border: Color::hsla(120.0, 0.50, 0.35, 1.0),
    text_color: Color::hsla(120.0, 0.20, 0.85, 1.0),
};

/// Button style for the Disconnect button in wizard select — minimal, unobtrusive.
pub(super) const DISCONNECT_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 100.0,
    height: 34.0,
    border_width: 1.0,
    font_size: 13.0,
    background: Color::hsla(0.0, 0.0, 0.10, 1.0),
    border: Color::hsla(0.0, 0.0, 0.22, 1.0),
    text_color: Color::hsla(0.0, 0.0, 0.50, 1.0),
};
