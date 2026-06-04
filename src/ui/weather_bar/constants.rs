use bevy::prelude::*;

/// Width of each weather button.
pub(super) const BUTTON_WIDTH: f32 = 64.0;

/// Height of each weather button. Kept small so the bar fits the gap above the
/// cast/mana bars without the buttons' top edge being clipped.
pub(super) const BUTTON_HEIGHT: f32 = 28.0;

/// Gap between weather buttons.
pub(super) const BUTTON_GAP: f32 = 5.0;

/// Border width for weather buttons.
pub(super) const BUTTON_BORDER_WIDTH: f32 = 2.0;

/// Font size for weather button labels.
pub(super) const LABEL_FONT_SIZE: f32 = 8.0;

/// Font size for the hotkey indicator.
pub(super) const HOTKEY_FONT_SIZE: f32 = 7.0;

/// Font size for the status text.
pub(super) const STATUS_FONT_SIZE: f32 = 8.0;

/// Inactive button background.
pub(super) const INACTIVE_BG: Color = Color::srgba(0.1, 0.1, 0.1, 0.6);

/// Active button border highlight.
pub(super) const ACTIVE_BORDER: Color = Color::srgba(1.0, 1.0, 1.0, 0.8);

/// Inactive button border.
pub(super) const INACTIVE_BORDER: Color = Color::srgba(0.3, 0.3, 0.3, 0.5);

/// Hovered button border color.
pub(super) const HOVERED_BORDER: Color = Color::srgba(0.6, 0.6, 0.6, 0.8);

/// Intensity fill height (percentage of button height).
pub(super) const INTENSITY_FILL_MAX_HEIGHT: f32 = BUTTON_HEIGHT - BUTTON_BORDER_WIDTH * 2.0;

/// Status text color (vivid cyan-white).
pub(super) const STATUS_TEXT_COLOR: Color = Color::srgba(0.6, 0.9, 1.0, 1.0);

/// Maximum width for the status text to prevent bleeding under mana bar.
pub(super) const STATUS_TEXT_MAX_WIDTH: f32 = 320.0;

/// Bottom margin for the weather bar. The Meteorologist still has the action bar
/// (bottom 15px + 50px slots = top at ~65px), so the weather bar must sit ABOVE it
/// — otherwise the action bar renders over the weather buttons and clips their top.
pub(super) const BAR_BOTTOM_MARGIN: f32 = 70.0;
