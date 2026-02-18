//! Progress screen constants.

use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

// Colors
pub(super) const TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.9, 1.0);
pub(super) const UNLOCKED_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);
pub(super) const LOCKED_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);
pub(super) const COMPLETED_COLOR: Color = Color::srgb(0.3, 0.85, 0.3);
pub(super) const DESCRIPTION_COLOR: Color = Color::srgb(0.6, 0.6, 0.6);
pub(super) const INSIGHT_COLOR: Color = Color::srgb(0.5, 0.7, 1.0);
pub(super) const IN_PROGRESS_COLOR: Color = Color::srgb(0.5, 0.65, 0.9);
pub(super) const SECTION_BG: Color = Color::srgba(0.1, 0.1, 0.12, 0.8);

// Font sizes
pub(super) const TITLE_FONT_SIZE: f32 = 40.0;

pub(super) const ITEM_NAME_FONT_SIZE: f32 = 16.0;
pub(super) const ITEM_DESC_FONT_SIZE: f32 = 13.0;
pub(super) const STAT_VALUE_FONT_SIZE: f32 = 22.0;
pub(super) const STAT_LABEL_COLOR: Color = Color::hsla(0.0, 0.0, 0.55, 1.0);

// Layout
pub(super) const MARGIN: f32 = 16.0;
pub(super) const MARGIN_SMALL: f32 = 8.0;
pub(super) const SECTION_PADDING: f32 = 12.0;
pub(super) const COLUMN_GAP: f32 = 12.0;
pub(super) const COLUMN_TITLE_FONT_SIZE: f32 = 20.0;

// Button style
pub(super) const BUTTON_COLOR: Color = Color::hsla(0.0, 0.0, 0.15, 1.0);
pub(super) const BUTTON_BORDER: Color = Color::hsla(0.0, 0.0, 0.3, 1.0);

pub(super) const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 200.0,
    height: 50.0,
    border_width: 3.0,
    font_size: 24.0,
    background: BUTTON_COLOR,
    border: BUTTON_BORDER,
    text_color: TEXT_COLOR,
};

// Danger button style (red)
pub(super) const DANGER_BUTTON_BACKGROUND: Color = Color::hsla(0.0, 0.7, 0.3, 1.0);
pub(super) const DANGER_BUTTON_BORDER: Color = Color::hsla(0.0, 0.8, 0.5, 1.0);

pub(super) const DANGER_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 200.0,
    height: 50.0,
    border_width: 3.0,
    font_size: 20.0,
    background: DANGER_BUTTON_BACKGROUND,
    border: DANGER_BUTTON_BORDER,
    text_color: TEXT_COLOR,
};
