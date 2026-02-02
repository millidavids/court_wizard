use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

/// Width of each action bar slot button.
pub(super) const SLOT_WIDTH: f32 = 50.0;

/// Height of each action bar slot button.
pub(super) const SLOT_HEIGHT: f32 = 50.0;

/// Gap between action bar slots.
pub(super) const SLOT_GAP: f32 = 4.0;

/// Font size for spell name text in action bar slots.
pub(super) const SPELL_NAME_FONT_SIZE: f32 = 10.0;

/// Font size for hotkey indicator text.
pub(super) const HOTKEY_FONT_SIZE: f32 = 10.0;

/// Button style for action bar slots.
pub(super) const SLOT_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: SLOT_WIDTH,
    height: SLOT_HEIGHT,
    border_width: 2.0,
    font_size: SPELL_NAME_FONT_SIZE,
    background: Color::srgba(0.15, 0.15, 0.15, 0.9),
    border: Color::srgba(0.4, 0.4, 0.4, 1.0),
    text_color: Color::WHITE,
};

/// Bottom margin for the action bar from screen edge.
pub(super) const ACTION_BAR_BOTTOM_MARGIN: f32 = 20.0;

/// Left margin for the action bar from screen edge.
pub(super) const ACTION_BAR_LEFT_MARGIN: f32 = 20.0;
