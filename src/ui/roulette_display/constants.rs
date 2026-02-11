use bevy::prelude::*;

/// Bottom margin from screen edge (same position as rune display).
pub(super) const BOTTOM_MARGIN: f32 = 20.0;

/// Radius of the roulette wheel (outer edge).
pub(super) const WHEEL_RADIUS: f32 = 40.0;

/// Font size for the selected spell name display above the wheel.
pub(super) const SELECTED_SPELL_FONT_SIZE: f32 = 22.0;

/// Font size for the "Press SPACE" prompt.
pub(super) const PROMPT_FONT_SIZE: f32 = 16.0;

/// Color for the "Press SPACE" prompt.
pub(super) const PROMPT_COLOR: Color = Color::srgba(0.8, 0.8, 0.9, 1.0);

/// Color for the selected spell name display.
pub(super) const SELECTED_SPELL_COLOR: Color = Color::srgba(1.0, 0.85, 0.3, 1.0);

/// Color for the pointer/indicator at the top of the wheel.
pub(super) const POINTER_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);

/// Duration in seconds for the selected spell name to fade out.
pub(super) const SELECTED_FADE_DURATION: f32 = 2.0;
