use bevy::prelude::*;

/// Bottom margin for the concentration UI from screen edge (just above action bar).
pub(super) const CONCENTRATION_UI_BOTTOM_MARGIN: f32 = 90.0;

/// Height of the concentration UI button.
pub(super) const CONCENTRATION_UI_HEIGHT: f32 = 40.0;

/// Font size for "End Concentration" button text.
pub(super) const BUTTON_FONT_SIZE: f32 = 9.0;

/// Background color for "End Concentration" button.
pub(super) const BUTTON_BACKGROUND: Color = Color::srgba(0.6, 0.2, 0.2, 0.9);

/// Hover background color for "End Concentration" button.
pub(super) const BUTTON_HOVER: Color = Color::srgba(0.8, 0.3, 0.3, 0.9);

/// Border color for "End Concentration" button.
pub(super) const BUTTON_BORDER: Color = Color::srgba(0.8, 0.3, 0.3, 1.0);

/// Text color for "End Concentration" button.
pub(super) const BUTTON_TEXT_COLOR: Color = Color::WHITE;
