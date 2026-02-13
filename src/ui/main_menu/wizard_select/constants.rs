//! Wizard select screen styling constants.

use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size for the wizard select title text.
pub(super) const TITLE_FONT_SIZE: f32 = 38.0;

/// Font size for the subtitle text.
pub(super) const SUBTITLE_FONT_SIZE: f32 = 13.0;

/// Text color for primary headings.
pub(super) const TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.92, 1.0);

/// Subdued text color for secondary elements.
pub(super) const SUBTITLE_COLOR: Color = Color::hsla(0.0, 0.0, 0.45, 1.0);

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

/// Margin between wizard select screen UI elements in pixels.
pub(super) const MARGIN: f32 = 20.0;

/// Total number of grid slots (4x4).
pub(super) const GRID_SLOTS: usize = 16;

/// Gap between grid cards in pixels.
pub(super) const CARD_GAP: f32 = 8.0;

/// Width of the left panel (title + detail + back).
pub(super) const LEFT_PANEL_WIDTH: f32 = 300.0;

// ---------------------------------------------------------------------------
// Grid cards (small)
// ---------------------------------------------------------------------------

/// Number of columns in the wizard grid.
pub(super) const GRID_COLUMNS: usize = 4;

/// Card width in pixels.
pub(super) const CARD_WIDTH: f32 = 210.0;

/// Card height in pixels.
pub(super) const CARD_HEIGHT: f32 = 140.0;

/// Card border width in pixels.
pub(super) const CARD_BORDER_WIDTH: f32 = 1.0;

/// Card border radius in pixels.
pub(super) const CARD_BORDER_RADIUS: f32 = 4.0;

/// Font size for wizard name on cards.
pub(super) const CARD_NAME_FONT_SIZE: f32 = 18.0;

/// Font size for wizard description on cards.
pub(super) const CARD_DESC_FONT_SIZE: f32 = 12.0;

/// Background color for unlocked wizard cards.
pub(super) const CARD_BG: Color = Color::hsla(220.0, 0.08, 0.11, 1.0);

/// Border color for unlocked wizard cards.
pub(super) const CARD_BORDER: Color = Color::hsla(0.0, 0.0, 0.20, 1.0);

/// Border color for the selected/active wizard card — gold accent.
pub(super) const CARD_BORDER_SELECTED: Color = Color::hsla(40.0, 0.50, 0.45, 1.0);

/// Color for wizard type short description text on cards.
pub(super) const DESCRIPTION_COLOR: Color = Color::hsla(0.0, 0.0, 0.45, 1.0);

/// Accent color for wizard name text — slightly warm white.
pub(super) const CARD_NAME_COLOR: Color = Color::hsla(40.0, 0.10, 0.85, 1.0);

// ---------------------------------------------------------------------------
// Detail panel (large, left side)
// ---------------------------------------------------------------------------

/// Detail panel border width.
pub(super) const DETAIL_BORDER_WIDTH: f32 = 1.0;

/// Detail panel border radius.
pub(super) const DETAIL_BORDER_RADIUS: f32 = 6.0;

/// Detail panel background color.
pub(super) const DETAIL_BG: Color = Color::hsla(220.0, 0.08, 0.10, 1.0);

/// Detail panel border color — gold accent.
pub(super) const DETAIL_BORDER: Color = Color::hsla(40.0, 0.35, 0.30, 1.0);

/// Font size for the wizard name in the detail panel.
pub(super) const DETAIL_NAME_FONT_SIZE: f32 = 24.0;

/// Font size for the long description in the detail panel.
pub(super) const DETAIL_DESC_FONT_SIZE: f32 = 12.0;

/// Color for the long description text.
pub(super) const DETAIL_DESC_COLOR: Color = Color::hsla(0.0, 0.0, 0.58, 1.0);

/// Font size for status text in the detail panel.
pub(super) const DETAIL_STATUS_FONT_SIZE: f32 = 13.0;

/// Color for stat text — warm amber.
pub(super) const STAT_COLOR: Color = Color::hsla(40.0, 0.55, 0.55, 1.0);

/// Color for "New" indicator text — soft green.
pub(super) const NEW_COLOR: Color = Color::hsla(140.0, 0.40, 0.50, 1.0);

// ---------------------------------------------------------------------------
// Locked cards
// ---------------------------------------------------------------------------

/// Background color for locked (unavailable) wizard cards.
pub(super) const LOCKED_CARD_BG: Color = Color::hsla(220.0, 0.05, 0.065, 1.0);

/// Border color for locked wizard cards.
pub(super) const LOCKED_CARD_BORDER: Color = Color::hsla(220.0, 0.05, 0.12, 1.0);

/// Text color for locked wizard cards.
pub(super) const LOCKED_TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.20, 1.0);

// ---------------------------------------------------------------------------
// Buttons
// ---------------------------------------------------------------------------

/// Button style for the Play button in the detail panel.
pub(super) const PLAY_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 140.0,
    height: 36.0,
    border_width: 1.0,
    font_size: 15.0,
    background: Color::hsla(40.0, 0.20, 0.18, 1.0),
    border: Color::hsla(40.0, 0.40, 0.35, 1.0),
    text_color: Color::hsla(40.0, 0.20, 0.85, 1.0),
};

/// Button style for the Back button — minimal, unobtrusive.
pub(super) const BACK_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 100.0,
    height: 34.0,
    border_width: 1.0,
    font_size: 13.0,
    background: Color::hsla(0.0, 0.0, 0.10, 1.0),
    border: Color::hsla(0.0, 0.0, 0.22, 1.0),
    text_color: Color::hsla(0.0, 0.0, 0.50, 1.0),
};
