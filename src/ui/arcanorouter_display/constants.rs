use bevy::color::Color;

// Layout dimensions - much smaller
pub(super) const SLIDER_HEIGHT: f32 = 80.0;
pub(super) const SLIDER_WIDTH: f32 = 20.0;
pub(super) const SLIDER_GAP: f32 = 10.0;

// Positioning - at very bottom of screen
pub(super) const BOTTOM_OFFSET: f32 = 10.0; // Distance from bottom of screen

// Colors - no background, muted dark colors
pub(super) const BACKGROUND_COLOR: Color = Color::NONE;
pub(super) const BAR_BACKGROUND_COLOR: Color = Color::srgba(0.2, 0.2, 0.25, 0.5);

// Slider colors by type - muted and darker
pub(super) const RANGE_COLOR: Color = Color::srgba(0.2, 0.3, 0.5, 0.8); // Muted dark blue
pub(super) const MANA_COLOR: Color = Color::srgba(0.4, 0.2, 0.5, 0.8); // Muted dark purple
pub(super) const POWER_COLOR: Color = Color::srgba(0.5, 0.2, 0.2, 0.8); // Muted dark red
pub(super) const SPEED_COLOR: Color = Color::srgba(0.2, 0.4, 0.2, 0.8); // Muted dark green

// Text - smaller fonts
pub(super) const LABEL_FONT_SIZE: f32 = 9.0;
pub(super) const TEXT_COLOR: Color = Color::WHITE;
