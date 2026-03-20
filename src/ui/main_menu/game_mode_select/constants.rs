use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

/// Text color for the game mode select screen.
pub(super) const TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.9, 1.0);

/// Title font size.
pub(super) const TITLE_FONT_SIZE: f32 = 36.0;

/// Subtitle font size.
pub(super) const SUBTITLE_FONT_SIZE: f32 = 12.0;

/// Subtitle color.
pub(super) const SUBTITLE_COLOR: Color = Color::hsla(0.0, 0.0, 0.5, 1.0);

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
    background: Color::hsla(0.0, 0.0, 0.15, 1.0),
    border: Color::hsla(0.0, 0.0, 0.3, 1.0),
    text_color: TEXT_COLOR,
};

/// Button style for disabled (coming soon) game modes.
pub(super) const DISABLED_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 200.0,
    height: 80.0,
    border_width: 3.0,
    font_size: 18.0,
    background: Color::hsla(0.0, 0.0, 0.08, 1.0),
    border: Color::hsla(0.0, 0.0, 0.18, 1.0),
    text_color: Color::hsla(0.0, 0.0, 0.35, 1.0),
};

/// Button style for the back button.
pub(super) const BACK_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 200.0,
    height: 60.0,
    border_width: 3.0,
    font_size: 20.0,
    background: Color::hsla(0.0, 0.0, 0.15, 1.0),
    border: Color::hsla(0.0, 0.0, 0.3, 1.0),
    text_color: TEXT_COLOR,
};
