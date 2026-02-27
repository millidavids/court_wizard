use bevy::prelude::*;

/// Delay before the fade-in begins (seconds).
pub(super) const DELAY_DURATION: f32 = 0.5;

/// How long elements take to fade in (seconds).
pub(super) const FADE_IN_DURATION: f32 = 1.0;

/// How long elements stay at full opacity (seconds).
pub(super) const HOLD_DURATION: f32 = 1.5;

/// How long elements take to fade out (seconds).
pub(super) const FADE_OUT_DURATION: f32 = 1.0;

/// Studio name text color (at full opacity).
pub(super) const STUDIO_TEXT_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);

/// Font size for "The Cult of" text.
pub(super) const TEXT_FONT_SIZE: f32 = 64.0;

/// Font size for "David" text (2x the base).
pub(super) const DAVID_FONT_SIZE: f32 = 128.0;

/// Maximum opacity for the studio background image.
pub(super) const STUDIO_IMAGE_MAX_OPACITY: f32 = 0.05;

/// Asset path for the studio background image.
pub(super) const STUDIO_IMAGE_PATH: &str = "images/studio.png";

/// Asset path for the Rust language logo.
pub(super) const RUST_LOGO_PATH: &str = "images/rust-logo-256x256-blk.png";

/// Asset path for the Bevy engine logo.
pub(super) const BEVY_LOGO_PATH: &str = "images/bevy_logo_dark.png";

/// Size of the Rust logo image.
pub(super) const RUST_LOGO_SIZE: f32 = 480.0;

/// Size of the gray background circle behind the Rust logo.
pub(super) const RUST_CIRCLE_SIZE: f32 = 560.0;

/// Color of the gray background circle behind the Rust logo.
pub(super) const RUST_CIRCLE_COLOR: Color = Color::srgb(0.35, 0.35, 0.35);

/// Height of the Bevy logo image.
pub(super) const BEVY_LOGO_HEIGHT: f32 = 200.0;
