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

// Wizard select constants (previously shared from wizard_select::shared)
use crate::ui::constants::{DETAIL_BG, DETAIL_BORDER, GOLD_ACCENT, TEXT_PRIMARY};

pub(super) const TITLE_FONT_SIZE: f32 = 29.0;
pub(super) const SUBTITLE_FONT_SIZE: f32 = 10.0;
pub(super) const TEXT_COLOR: Color = TEXT_PRIMARY;
pub(super) const SUBTITLE_COLOR: Color = TEXT_MUTED;
pub(super) const LEFT_PANEL_WIDTH: f32 = 300.0;
pub(super) const MARGIN: f32 = 20.0;
pub(super) const GRID_SLOTS: usize = 16;
pub(super) const CARD_GAP: f32 = 8.0;
pub(super) const GRID_COLUMNS: usize = 4;
pub(super) const CARD_WIDTH: f32 = 210.0;
pub(super) const CARD_HEIGHT: f32 = 140.0;
pub(super) const CARD_BORDER_WIDTH: f32 = 1.0;
pub(super) const CARD_BORDER_RADIUS: f32 = 4.0;
pub(super) const CARD_NAME_FONT_SIZE: f32 = 14.0;
pub(super) const CARD_DESC_FONT_SIZE: f32 = 10.0;
pub(super) const CARD_BG: Color = Color::hsla(220.0, 0.08, 0.11, 0.75);
pub(super) const CARD_BORDER: Color = Color::hsla(0.0, 0.0, 0.20, 0.6);
pub(super) const CARD_BORDER_SELECTED: Color = GOLD_ACCENT;
pub(super) const DESCRIPTION_COLOR: Color = TEXT_MUTED;
pub(super) const CARD_NAME_COLOR: Color = Color::hsla(40.0, 0.10, 0.85, 1.0);
pub(super) const DETAIL_BORDER_WIDTH: f32 = 1.0;
pub(super) const DETAIL_BORDER_RADIUS: f32 = 6.0;
pub(super) const DETAIL_PANEL_BG: Color = DETAIL_BG;
pub(super) const DETAIL_PANEL_BORDER: Color = DETAIL_BORDER;
pub(super) const DETAIL_NAME_FONT_SIZE: f32 = 18.0;
pub(super) const DETAIL_DESC_FONT_SIZE: f32 = 10.0;
pub(super) const DETAIL_DESC_COLOR: Color = Color::hsla(0.0, 0.0, 0.58, 1.0);
pub(super) const DETAIL_STATUS_FONT_SIZE: f32 = 10.0;
pub(super) const LOCKED_CARD_BG: Color = Color::hsla(20.0, 0.08, 0.06, 0.6);
pub(super) const LOCKED_CARD_BORDER: Color = Color::hsla(25.0, 0.10, 0.12, 0.5);
pub(super) const LOCKED_TEXT_COLOR: Color = Color::hsla(30.0, 0.06, 0.45, 1.0);
pub(super) const SEPARATOR_COLOR: Color = Color::hsla(270.0, 0.10, 0.15, 1.0);

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
