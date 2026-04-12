use bevy::prelude::*;

use crate::ui::constants::{TEXT_BODY, TEXT_MUTED};

/// How long the popup stays at full opacity (seconds).
pub(super) const DISPLAY_DURATION: f32 = 4.0;

/// How long the popup takes to fade out (seconds).
pub(super) const FADE_DURATION: f32 = 1.0;

/// Background color of the popup box.
pub(super) const BACKGROUND_COLOR: Color = Color::hsla(20.0, 0.12, 0.07, 0.92);

/// Border color (golden).
pub(super) const BORDER_COLOR: Color = Color::hsla(42.0, 0.65, 0.45, 1.0);

/// Achievement name text color.
pub(super) const TITLE_COLOR: Color = Color::hsla(270.0, 0.70, 0.75, 1.0);

/// Achievement description text color.
pub(super) const DESCRIPTION_COLOR: Color = TEXT_BODY;

/// "Achievement Unlocked" header color.
pub(super) const HEADER_COLOR: Color = TEXT_MUTED;

// ===== Ingredient Popup Colors =====

/// Background color of the ingredient popup box.
pub(super) const INGREDIENT_BACKGROUND_COLOR: Color = Color::srgba(0.06, 0.12, 0.06, 0.92);

/// Border color for ingredient popups (green).
pub(super) const INGREDIENT_BORDER_COLOR: Color = Color::srgb(0.4, 0.9, 0.4);

/// "Ingredient Discovered!" header color.
pub(super) const INGREDIENT_HEADER_COLOR: Color = Color::srgb(0.4, 0.9, 0.4);

/// Ingredient name text color.
pub(super) const INGREDIENT_TITLE_COLOR: Color = Color::srgb(0.85, 0.95, 0.85);

// ===== Spell Researched Popup Colors =====

/// Background color of the spell researched popup.
pub(super) const SPELL_BACKGROUND_COLOR: Color = Color::hsla(225.0, 0.20, 0.08, 0.92);

/// Border color for spell researched popups (blue/arcane).
pub(super) const SPELL_BORDER_COLOR: Color = Color::srgb(0.4, 0.6, 1.0);

/// "Spell Researched!" header color.
pub(super) const SPELL_HEADER_COLOR: Color = Color::srgb(0.5, 0.7, 1.0);

/// Spell name text color.
pub(super) const SPELL_TITLE_COLOR: Color = Color::srgb(0.85, 0.9, 1.0);

// ===== Combo Discovered Popup Colors =====

/// Background color of the combo discovered popup.
pub(super) const COMBO_BACKGROUND_COLOR: Color = Color::hsla(270.0, 0.10, 0.08, 0.92);

/// Border color for combo popups (purple).
pub(super) const COMBO_BORDER_COLOR: Color = Color::hsla(270.0, 0.65, 0.50, 1.0);

/// "Combo Discovered!" header color.
pub(super) const COMBO_HEADER_COLOR: Color = Color::hsla(270.0, 0.55, 0.60, 1.0);

/// Combo name text color (purple).
pub(super) const COMBO_TITLE_COLOR: Color = Color::hsla(270.0, 0.70, 0.75, 1.0);
