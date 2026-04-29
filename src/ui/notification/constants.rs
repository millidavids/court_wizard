use bevy::prelude::*;

use crate::ui::constants::TEXT_BODY;

/// How long the notification stays at full opacity (seconds).
pub(super) const DISPLAY_DURATION: f32 = 3.0;

/// How long the notification takes to fade out (seconds).
pub(super) const FADE_DURATION: f32 = 0.8;

/// Background color of the default toast box (used by settings confirmation toasts).
pub(super) const BACKGROUND_COLOR: Color = Color::hsla(20.0, 0.12, 0.07, 0.92);

/// Default toast border color.
pub(super) const BORDER_COLOR: Color = Color::hsla(42.0, 0.65, 0.45, 1.0);

/// Default toast description text color (also used as title color for plain toasts).
pub(super) const DESCRIPTION_COLOR: Color = TEXT_BODY;

// ===== Ingredient Notification Colors =====

pub(super) const INGREDIENT_BACKGROUND_COLOR: Color = Color::srgba(0.06, 0.12, 0.06, 0.92);
pub(super) const INGREDIENT_BORDER_COLOR: Color = Color::srgb(0.4, 0.9, 0.4);
pub(super) const INGREDIENT_HEADER_COLOR: Color = Color::srgb(0.4, 0.9, 0.4);
pub(super) const INGREDIENT_TITLE_COLOR: Color = Color::srgb(0.85, 0.95, 0.85);

// ===== Spell Researched Notification Colors =====

pub(super) const SPELL_BACKGROUND_COLOR: Color = Color::hsla(225.0, 0.20, 0.08, 0.92);
pub(super) const SPELL_BORDER_COLOR: Color = Color::srgb(0.4, 0.6, 1.0);
pub(super) const SPELL_HEADER_COLOR: Color = Color::srgb(0.5, 0.7, 1.0);
pub(super) const SPELL_TITLE_COLOR: Color = Color::srgb(0.85, 0.9, 1.0);

// ===== Combo Discovered Notification Colors =====

pub(super) const COMBO_BACKGROUND_COLOR: Color = Color::hsla(270.0, 0.10, 0.08, 0.92);
pub(super) const COMBO_BORDER_COLOR: Color = Color::hsla(270.0, 0.65, 0.50, 1.0);
pub(super) const COMBO_HEADER_COLOR: Color = Color::hsla(270.0, 0.55, 0.60, 1.0);
pub(super) const COMBO_TITLE_COLOR: Color = Color::hsla(270.0, 0.70, 0.75, 1.0);

// ===== Wizard Unlocked Notification Colors (purple) =====

pub(super) const WIZARD_BACKGROUND_COLOR: Color = Color::hsla(270.0, 0.15, 0.07, 0.92);
pub(super) const WIZARD_BORDER_COLOR: Color = Color::hsla(270.0, 0.65, 0.50, 1.0);
pub(super) const WIZARD_HEADER_COLOR: Color = Color::hsla(270.0, 0.55, 0.70, 1.0);
pub(super) const WIZARD_TITLE_COLOR: Color = Color::hsla(270.0, 0.70, 0.80, 1.0);

// ===== Talent Tier Notification Colors (gold) =====

pub(super) const TALENT_BACKGROUND_COLOR: Color = Color::hsla(42.0, 0.15, 0.07, 0.92);
pub(super) const TALENT_BORDER_COLOR: Color = Color::hsla(42.0, 0.65, 0.50, 1.0);
pub(super) const TALENT_HEADER_COLOR: Color = Color::hsla(42.0, 0.65, 0.60, 1.0);
pub(super) const TALENT_TITLE_COLOR: Color = Color::hsla(42.0, 0.55, 0.80, 1.0);
