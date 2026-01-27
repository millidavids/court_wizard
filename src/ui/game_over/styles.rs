use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

pub const BACKGROUND_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.85);
pub const TITLE_COLOR: Color = Color::srgb(0.95, 0.95, 0.95);
pub const TEXT_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);

pub const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 250.0,
    height: 65.0,
    border_width: 3.0,
    font_size: 28.0,
    background: Color::hsla(0.0, 0.0, 0.15, 1.0),
    border: Color::hsla(0.0, 0.0, 0.3, 1.0),
    text_color: Color::hsla(0.0, 0.0, 0.9, 1.0),
};
