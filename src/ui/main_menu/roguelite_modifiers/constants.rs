use bevy::prelude::*;

use crate::ui::components::ButtonStyle;
use crate::ui::constants::{BUTTON_BG, BUTTON_BORDER, TEXT_MUTED, TEXT_PRIMARY};

/// Text color for modifier labels.
pub(super) const TEXT_COLOR: Color = TEXT_PRIMARY;

/// Subtitle / description color.
pub(super) const DESCRIPTION_COLOR: Color = TEXT_MUTED;

/// Title font size.
pub(super) const TITLE_FONT_SIZE: f32 = 32.0;

/// Subtitle font size.
pub(super) const SUBTITLE_FONT_SIZE: f32 = 11.0;

/// Spacing between sections.
pub(super) const MARGIN: f32 = 20.0;

/// Smaller spacing inside slider rows.
pub(super) const MARGIN_SMALL: f32 = 10.0;

/// Continue button style.
pub(super) const CONTINUE_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 200.0,
    height: 60.0,
    border_width: 3.0,
    font_size: 20.0,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};

/// Back button style.
pub(super) const BACK_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 200.0,
    height: 60.0,
    border_width: 3.0,
    font_size: 20.0,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};

/// Reset button style.
pub(super) const RESET_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 140.0,
    height: 40.0,
    border_width: 2.0,
    font_size: 12.0,
    background: Color::hsla(0.0, 0.0, 0.15, 1.0),
    border: Color::hsla(0.0, 0.0, 0.4, 1.0),
    text_color: TEXT_MUTED,
    text_shadow: false,
};
