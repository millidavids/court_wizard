//! Shared visual constants and button styles for the multiplayer tab panels.

use bevy::prelude::*;

use crate::ui::components::ButtonStyle;
use crate::ui::constants::{
    BUTTON_BG, BUTTON_BG_SUBTLE, BUTTON_BORDER, BUTTON_BORDER_SUBTLE, TEXT_PRIMARY,
};

pub(super) const SECTION_FONT_SIZE: f32 = 13.0;
pub(super) const BODY_FONT_SIZE: f32 = 12.0;
pub(super) const HINT_FONT_SIZE: f32 = 11.0;
pub(super) const HEADING_FONT_SIZE: f32 = 16.0;
pub(super) const CODE_FONT_SIZE: f32 = 10.0;

/// Inner padding for the right panel content (the right panel node itself has none).
pub(super) const PANEL_PADDING: f32 = 16.0;

pub(super) const CARD_BG: Color = Color::hsla(220.0, 0.08, 0.11, 0.75);
pub(super) const CARD_BORDER: Color = Color::hsla(0.0, 0.0, 0.20, 0.6);
pub(super) const CARD_BORDER_SELECTED: Color = crate::ui::constants::GOLD_ACCENT;
pub(super) const CARD_BORDER_WIDTH: f32 = 1.0;
pub(super) const CARD_BORDER_RADIUS: f32 = 4.0;

pub(super) const CODE_BOX_BG: Color = Color::hsla(270.0, 0.08, 0.08, 1.0);
pub(super) const CODE_BOX_BORDER_FOCUSED: Color = Color::hsla(270.0, 0.65, 0.55, 1.0);
pub(super) const CODE_BOX_BORDER_UNFOCUSED: Color = Color::hsla(270.0, 0.35, 0.35, 1.0);

pub(super) const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 220.0,
    height: 44.0,
    border_width: 2.0,
    font_size: 14.0,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};

/// Compact button used inline beside the code box (Copy / Paste).
pub(super) const INLINE_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 84.0,
    height: 40.0,
    border_width: 2.0,
    font_size: 11.0,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};

pub(super) const SMALL_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 180.0,
    height: 36.0,
    border_width: 1.0,
    font_size: 12.0,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};

pub(super) const READY_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 180.0,
    height: 40.0,
    border_width: 2.0,
    font_size: 14.0,
    background: Color::hsla(120.0, 0.30, 0.20, 0.75),
    border: Color::hsla(120.0, 0.50, 0.35, 1.0),
    text_color: Color::hsla(120.0, 0.20, 0.85, 1.0),
    text_shadow: true,
};

pub(super) const UNREADY_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 180.0,
    height: 40.0,
    border_width: 2.0,
    font_size: 14.0,
    background: Color::hsla(0.0, 0.30, 0.20, 0.75),
    border: Color::hsla(0.0, 0.50, 0.35, 1.0),
    text_color: Color::hsla(0.0, 0.20, 0.85, 1.0),
    text_shadow: true,
};

pub(super) const DISCONNECT_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 180.0,
    height: 36.0,
    border_width: 1.0,
    font_size: 12.0,
    background: BUTTON_BG_SUBTLE,
    border: BUTTON_BORDER_SUBTLE,
    text_color: Color::hsla(0.0, 0.0, 0.70, 1.0),
    text_shadow: false,
};
