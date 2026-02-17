use bevy::prelude::*;

/// How long the popup stays at full opacity (seconds).
pub(super) const DISPLAY_DURATION: f32 = 4.0;

/// How long the popup takes to fade out (seconds).
pub(super) const FADE_DURATION: f32 = 1.0;

/// Background color of the popup box.
pub(super) const BACKGROUND_COLOR: Color = Color::srgba(0.08, 0.06, 0.12, 0.92);

/// Border color (gold/amber).
pub(super) const BORDER_COLOR: Color = Color::srgb(0.85, 0.65, 0.13);

/// Achievement name text color.
pub(super) const TITLE_COLOR: Color = Color::srgb(0.95, 0.82, 0.30);

/// Achievement description text color.
pub(super) const DESCRIPTION_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);

/// "Achievement Unlocked" header color.
pub(super) const HEADER_COLOR: Color = Color::srgb(0.70, 0.70, 0.70);

// ===== Ingredient Popup Colors =====

/// Background color of the ingredient popup box.
pub(super) const INGREDIENT_BACKGROUND_COLOR: Color = Color::srgba(0.06, 0.12, 0.06, 0.92);

/// Border color for ingredient popups (green).
pub(super) const INGREDIENT_BORDER_COLOR: Color = Color::srgb(0.4, 0.9, 0.4);

/// "Ingredient Discovered!" header color.
pub(super) const INGREDIENT_HEADER_COLOR: Color = Color::srgb(0.4, 0.9, 0.4);

/// Ingredient name text color.
pub(super) const INGREDIENT_TITLE_COLOR: Color = Color::srgb(0.85, 0.95, 0.85);
