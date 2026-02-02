use bevy::prelude::*;

/// Font size for the rune sequence display.
pub(super) const RUNE_SEQUENCE_FONT_SIZE: f32 = 24.0;

/// Font size for the validity indicator.
pub(super) const VALIDITY_FONT_SIZE: f32 = 16.0;

/// Bottom margin from screen edge.
pub(super) const BOTTOM_MARGIN: f32 = 80.0;

/// Right margin from screen edge.
pub(super) const RIGHT_MARGIN: f32 = 20.0;

/// Background color for the rune display.
pub(super) const BACKGROUND_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 0.8);

/// Border color for the rune display.
pub(super) const BORDER_COLOR: Color = Color::srgba(0.4, 0.4, 0.4, 1.0);

/// Text color for rune sequence.
pub(super) const SEQUENCE_TEXT_COLOR: Color = Color::srgba(0.9, 0.9, 1.0, 1.0);

/// Text color for valid sequence indicator.
pub(super) const VALID_COLOR: Color = Color::srgba(0.2, 1.0, 0.2, 1.0);

/// Text color for invalid sequence indicator.
pub(super) const INVALID_COLOR: Color = Color::srgba(1.0, 0.3, 0.3, 1.0);

/// Padding inside the rune display box.
pub(super) const PADDING: f32 = 12.0;

/// Minimum width of the rune display box.
pub(super) const MIN_WIDTH: f32 = 150.0;
