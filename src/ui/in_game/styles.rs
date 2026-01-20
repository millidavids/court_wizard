use bevy::prelude::*;

/// Screen margin for HUD elements (invisible padding from edges).
pub const HUD_MARGIN: Val = Val::Px(20.0);

/// Mana bar dimensions.
pub const MANA_BAR_WIDTH: Val = Val::Vw(33.33); // 1/3 of screen width
pub const MANA_BAR_HEIGHT: Val = Val::Px(20.0);

/// Mana bar colors.
pub const MANA_BAR_BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.5); // 50% translucent black background
pub const MANA_BAR_FILL_COLOR: Color = Color::srgba(0.2, 0.4, 1.0, 0.7); // 70% translucent blue
