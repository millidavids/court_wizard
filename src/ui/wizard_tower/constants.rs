use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

pub(super) const BACKGROUND_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.85);
pub(super) const TITLE_COLOR: Color = Color::srgb(0.95, 0.95, 0.95);
pub(super) const TEXT_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);

pub(super) const TITLE_FONT_SIZE: f32 = 40.0;
pub(super) const LEVEL_FONT_SIZE: f32 = 18.0;

pub(super) const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 250.0,
    height: 65.0,
    border_width: 3.0,
    font_size: 14.0,
    background: Color::hsla(0.0, 0.0, 0.15, 1.0),
    border: Color::hsla(0.0, 0.0, 0.3, 1.0),
    text_color: Color::hsla(0.0, 0.0, 0.9, 1.0),
};
