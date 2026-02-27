use bevy::prelude::*;

/// Delay before the fade-in begins (seconds).
pub(super) const DELAY_DURATION: f32 = 1.0;

/// How long the text takes to fade in (seconds).
pub(super) const FADE_IN_DURATION: f32 = 1.0;

/// How long the text stays at full opacity (seconds).
pub(super) const HOLD_DURATION: f32 = 2.0;

/// How long the text takes to fade out (seconds).
pub(super) const FADE_OUT_DURATION: f32 = 1.0;

/// Studio name text color (at full opacity).
pub(super) const TEXT_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);

/// Font size for "The Cult of" text.
pub(super) const TEXT_FONT_SIZE: f32 = 64.0;

/// Font size for "David" text (2x the base).
pub(super) const DAVID_FONT_SIZE: f32 = 128.0;

/// Maximum opacity for the background image.
pub(super) const IMAGE_MAX_OPACITY: f32 = 0.05;

/// Asset path for the studio background image.
pub(super) const STUDIO_IMAGE_PATH: &str = "images/studio.png";
