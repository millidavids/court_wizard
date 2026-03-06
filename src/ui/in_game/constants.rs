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
pub const CAST_BAR_BREWING_FILL_COLOR: Color = Color::srgba(0.4, 0.4, 0.4, 0.7); // 70% translucent gray

/// Button dimensions.
pub const BUTTON_WIDTH: f32 = 120.0;
pub const BUTTON_HEIGHT: f32 = 50.0;
pub const BUTTON_BORDER_WIDTH: f32 = 2.0;

/// Button colors.
pub const BUTTON_BACKGROUND: Color = Color::srgb(0.15, 0.15, 0.15);
pub const BUTTON_BORDER: Color = Color::srgb(0.4, 0.4, 0.4);
pub const BUTTON_TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
pub const BUTTON_FONT_SIZE: f32 = 9.0;

/// Boss health bar dimensions.
pub const BOSS_HEALTH_BAR_WIDTH: Val = Val::Vw(50.0); // 50% of screen width
pub const BOSS_HEALTH_BAR_HEIGHT: Val = Val::Px(30.0);
pub const BOSS_HEALTH_BAR_TOP_MARGIN: f32 = 15.0;

/// Boss health bar colors.
pub const BOSS_HEALTH_BAR_BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);
pub const BOSS_HEALTH_BAR_FILL_COLOR: Color = Color::srgba(0.8, 0.1, 0.1, 0.8);
pub const BOSS_HEALTH_BAR_BORDER_COLOR: Color = Color::srgba(0.5, 0.1, 0.1, 1.0);

/// Boss name font size.
pub const BOSS_NAME_FONT_SIZE: f32 = 12.0;

/// Boss health bar text font size.
pub const BOSS_HEALTH_TEXT_FONT_SIZE: f32 = 9.0;

/// Hag health bar section colors (per identity).
pub const HAG_JUSTINA_BAR_COLOR: Color = Color::srgba(0.9, 0.4, 0.1, 0.8);
pub const HAG_MARTINA_BAR_COLOR: Color = Color::srgba(0.5, 0.15, 0.7, 0.8);
pub const HAG_JOSEPHINA_BAR_COLOR: Color = Color::srgba(0.6, 0.35, 0.15, 0.8);

/// Gap between hag health bar sections.
pub const HAG_BAR_SECTION_GAP: f32 = 3.0;

/// Dimmed color for permanently dead hag sections.
pub const HAG_BAR_DEAD_COLOR: Color = Color::srgba(0.3, 0.3, 0.3, 0.4);

/// King health bar dimensions.
pub(super) const KING_HEALTH_BAR_WIDTH: Val = Val::Px(20.0);
pub(super) const KING_HEALTH_BAR_HEIGHT: Val = Val::Vh(30.0);

/// King health bar colors.
pub(super) const KING_HEALTH_BAR_BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.5);
pub(super) const KING_HEALTH_BAR_FILL_COLOR: Color = Color::srgba(0.2, 0.8, 0.2, 0.7);
pub(super) const KING_HEALTH_BAR_BORDER_COLOR: Color = Color::srgba(0.3, 0.6, 0.3, 0.8);

/// King health bar label font size.
pub(super) const KING_HEALTH_BAR_LABEL_FONT_SIZE: f32 = 7.0;

// ===== Level Clock Constants =====

/// Font size for the level clock display.
pub(super) const LEVEL_CLOCK_FONT_SIZE: f32 = 20.0;

/// Color for the level clock text.
pub(super) const LEVEL_CLOCK_COLOR: Color = Color::srgba(0.7, 0.8, 0.9, 0.9);

// ===== Wave Display Constants =====

/// Font size for the wave counter text.
pub(super) const WAVE_DISPLAY_FONT_SIZE: f32 = 22.0;

/// Color for the wave counter text.
pub(super) const WAVE_DISPLAY_COLOR: Color = Color::srgba(0.9, 0.9, 0.9, 0.9);

/// Font size for the "Wave incoming!" flash text.
pub(super) const WAVE_FLASH_FONT_SIZE: f32 = 28.0;

/// Color for the "Wave incoming!" flash text.
pub(super) const WAVE_FLASH_COLOR: Color = Color::srgb(1.0, 0.3, 0.3);

/// Duration the "Wave incoming!" flash is displayed (seconds).
pub(super) const WAVE_FLASH_DURATION: f32 = 3.0;

// ===== Buff Tracker Constants =====

/// Size of each buff tracker box.
pub(super) const BUFF_BOX_SIZE: f32 = 40.0;

/// Gap between buff tracker boxes.
pub(super) const BUFF_BOX_GAP: f32 = 6.0;

/// Border width for buff tracker boxes.
pub(super) const BUFF_BOX_BORDER_WIDTH: f32 = 1.0;

/// Font size for the abbreviation label in buff boxes.
pub(super) const BUFF_LABEL_FONT_SIZE: f32 = 10.0;

/// Font size for the timer text below the label.
pub(super) const BUFF_TIMER_FONT_SIZE: f32 = 7.0;

/// Border color for buff boxes.
pub(super) const BUFF_BOX_BORDER_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.3);

/// Buff tooltip background color.
pub(super) const BUFF_TOOLTIP_BG: Color = Color::srgba(0.05, 0.05, 0.05, 0.95);

/// Buff tooltip border color.
pub(super) const BUFF_TOOLTIP_BORDER: Color = Color::srgba(0.4, 0.4, 0.4, 0.8);

/// Buff tooltip font size.
pub(super) const BUFF_TOOLTIP_FONT_SIZE: f32 = 7.0;

/// Buff tooltip padding.
pub(super) const BUFF_TOOLTIP_PADDING: f32 = 8.0;

/// Buff tooltip max width.
pub(super) const BUFF_TOOLTIP_MAX_WIDTH: f32 = 200.0;

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
