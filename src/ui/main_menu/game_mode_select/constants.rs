use bevy::prelude::*;

use crate::ui::components::ButtonStyle;
use crate::ui::constants::{
    BUTTON_BG, BUTTON_BG_SUBTLE, BUTTON_BORDER, BUTTON_BORDER_SUBTLE, TEXT_DISABLED, TEXT_MUTED,
    TEXT_PRIMARY,
};

/// Text color (alias for global TEXT_PRIMARY).
pub(super) const TEXT_COLOR: Color = TEXT_PRIMARY;

/// Subtitle color.
pub(super) const SUBTITLE_COLOR: Color = TEXT_MUTED;

/// Title font size.
pub(super) const TITLE_FONT_SIZE: f32 = 36.0;

/// Subtitle font size.
pub(super) const SUBTITLE_FONT_SIZE: f32 = 12.0;

/// Margin between elements.
pub(super) const MARGIN: f32 = 20.0;

/// Grid gap between mode buttons.
pub(super) const GRID_GAP: f32 = 16.0;

/// Button style for active game modes.
pub(super) const MODE_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 200.0,
    height: 80.0,
    border_width: 3.0,
    font_size: 18.0,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};

/// Button style for disabled (coming soon) game modes.
pub(super) const DISABLED_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 200.0,
    height: 80.0,
    border_width: 3.0,
    font_size: 18.0,
    background: BUTTON_BG_SUBTLE,
    border: BUTTON_BORDER_SUBTLE,
    text_color: TEXT_DISABLED,
    text_shadow: true,
};

/// Button style for the back button.
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
