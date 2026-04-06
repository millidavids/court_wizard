use bevy::prelude::*;

/// How long each splash screen is displayed (seconds).
pub(super) const HOLD_DURATION: f32 = 4.0;

/// Asset path for the CRT effect shader (waited on during Black substate).
pub(super) const SHADER_ASSET_PATH: &str = "shaders/crt_effect.wgsl";

/// Asset path for the studio logo image.
pub(super) const STUDIO_IMAGE_PATH: &str = "images/logos/blackhearth_logo.png";

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
