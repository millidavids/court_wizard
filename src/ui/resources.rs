//! UI resources.

use bevy::prelude::*;

/// Resource holding the custom font asset handle with fallback support.
///
/// Loaded at startup and used throughout the game for all text rendering.
/// If the custom font fails to load, Bevy will automatically fall back to its
/// default font (FiraMono-subset.ttf) since the `default_font` feature is enabled.
#[derive(Resource)]
pub struct CustomFont {
    pub handle: Handle<Font>,
}
