//! Wizard select screen styling constants.
//!
//! Most styling is shared with the multiplayer wizard select via `wizard_select_shared`.
//! This file only contains single-player-specific constants.

use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

// Re-export all shared constants for convenience.
pub(super) use super::super::wizard_select_shared::*;

/// Color for stat text — warm amber.
pub(super) const STAT_COLOR: Color = Color::hsla(40.0, 0.55, 0.55, 1.0);

/// Color for "New" indicator text — soft green.
pub(super) const NEW_COLOR: Color = Color::hsla(140.0, 0.40, 0.50, 1.0);

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
