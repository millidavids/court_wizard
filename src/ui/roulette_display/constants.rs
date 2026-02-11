use bevy::prelude::*;

/// Bottom margin from screen edge (same position as rune display).
pub(super) const BOTTOM_MARGIN: f32 = 20.0;

/// Radius of the roulette wheel (outer edge).
pub(super) const WHEEL_RADIUS: f32 = 40.0;

/// Radius of the inner circle (center hole).
pub(super) const INNER_RADIUS: f32 = 15.0;

/// Font size for the selected spell name display above the wheel.
pub(super) const SELECTED_SPELL_FONT_SIZE: f32 = 22.0;

/// Font size for the "Press SPACE" prompt.
pub(super) const PROMPT_FONT_SIZE: f32 = 16.0;

/// Colorful wedge colors - vibrant rainbow palette.
pub(super) const WEDGE_COLORS: [Color; 12] = [
    Color::srgba(1.0, 0.2, 0.2, 0.95), // Red
    Color::srgba(1.0, 0.5, 0.0, 0.95), // Orange
    Color::srgba(1.0, 0.9, 0.0, 0.95), // Yellow
    Color::srgba(0.5, 1.0, 0.0, 0.95), // Yellow-Green
    Color::srgba(0.0, 1.0, 0.3, 0.95), // Green
    Color::srgba(0.0, 1.0, 0.8, 0.95), // Cyan
    Color::srgba(0.0, 0.7, 1.0, 0.95), // Light Blue
    Color::srgba(0.2, 0.3, 1.0, 0.95), // Blue
    Color::srgba(0.5, 0.0, 1.0, 0.95), // Purple
    Color::srgba(0.8, 0.0, 1.0, 0.95), // Magenta
    Color::srgba(1.0, 0.0, 0.6, 0.95), // Pink
    Color::srgba(1.0, 0.0, 0.3, 0.95), // Rose
];

/// Border color for the wheel.
pub(super) const WHEEL_BORDER: Color = Color::srgba(0.2, 0.2, 0.2, 1.0);

/// Color for the "Press SPACE" prompt.
pub(super) const PROMPT_COLOR: Color = Color::srgba(0.8, 0.8, 0.9, 1.0);

/// Color for the selected spell name display.
pub(super) const SELECTED_SPELL_COLOR: Color = Color::srgba(1.0, 0.85, 0.3, 1.0);

/// Color for the pointer/indicator at the top of the wheel.
pub(super) const POINTER_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 1.0);

/// Duration in seconds for the selected spell name to fade out.
pub(super) const SELECTED_FADE_DURATION: f32 = 2.0;
