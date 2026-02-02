use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

/// Width of each rune button.
pub(super) const RUNE_BUTTON_WIDTH: f32 = 50.0;

/// Height of each rune button.
pub(super) const RUNE_BUTTON_HEIGHT: f32 = 50.0;

/// Gap between rune buttons.
pub(super) const RUNE_BUTTON_GAP: f32 = 4.0;

/// Font size for rune letters on buttons.
pub(super) const RUNE_LETTER_FONT_SIZE: f32 = 24.0;

/// Font size for the rune sequence display above buttons.
pub(super) const RUNE_SEQUENCE_FONT_SIZE: f32 = 20.0;

/// Bottom margin from screen edge (between action bar and mana/cast bars).
pub(super) const BOTTOM_MARGIN: f32 = 20.0;

/// Button style for rune buttons.
pub(super) const RUNE_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: RUNE_BUTTON_WIDTH,
    height: RUNE_BUTTON_HEIGHT,
    border_width: 2.0,
    font_size: RUNE_LETTER_FONT_SIZE,
    background: Color::srgba(0.15, 0.15, 0.15, 0.9),
    border: Color::srgba(0.4, 0.4, 0.4, 1.0),
    text_color: Color::WHITE,
};

/// Text color for rune sequence display.
pub(super) const SEQUENCE_TEXT_COLOR: Color = Color::srgba(0.9, 0.9, 1.0, 1.0);

/// Duration in seconds for the spell name fade-out animation.
pub(super) const SPELL_NAME_FADE_DURATION: f32 = 2.0;
