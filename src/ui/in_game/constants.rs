use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

/// Screen margin for HUD elements (invisible padding from edges).
pub const HUD_MARGIN: Val = Val::Px(20.0);

/// Gap between HUD elements.
pub const HUD_ELEMENT_GAP: Val = Val::Px(10.0);

/// Mana bar dimensions.
pub const MANA_BAR_WIDTH: Val = Val::Vw(33.33); // 1/3 of screen width
pub const MANA_BAR_HEIGHT: Val = Val::Px(20.0);

/// Mana bar colors.
pub const MANA_BAR_BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.5); // 50% translucent black background
pub const MANA_BAR_FILL_COLOR: Color = Color::srgba(0.2, 0.4, 1.0, 0.7); // 70% translucent blue

/// Cast bar dimensions.
pub const CAST_BAR_WIDTH: Val = Val::Vw(33.33); // 1/3 of screen width
pub const CAST_BAR_HEIGHT: Val = Val::Px(15.0);

/// Cast bar colors.
pub const CAST_BAR_BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.5); // 50% translucent black background
pub const CAST_BAR_FILL_COLOR: Color = Color::srgba(1.0, 0.8, 0.0, 0.7); // 70% translucent yellow/gold

/// Button dimensions.
pub const BUTTON_WIDTH: f32 = 120.0;
pub const BUTTON_HEIGHT: f32 = 50.0;
pub const BUTTON_BORDER_WIDTH: f32 = 2.0;

/// Button colors.
pub const BUTTON_BACKGROUND: Color = Color::srgb(0.15, 0.15, 0.15);
pub const BUTTON_BORDER: Color = Color::srgb(0.4, 0.4, 0.4);
pub const BUTTON_TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
pub const BUTTON_FONT_SIZE: f32 = 24.0;

/// Button style configuration for the in-game HUD.
pub const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: BUTTON_WIDTH,
    height: BUTTON_HEIGHT,
    border_width: BUTTON_BORDER_WIDTH,
    font_size: BUTTON_FONT_SIZE,
    background: BUTTON_BACKGROUND,
    border: BUTTON_BORDER,
    text_color: BUTTON_TEXT_COLOR,
};
