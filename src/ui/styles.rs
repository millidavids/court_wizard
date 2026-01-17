//! Shared UI styling helpers.
//!
//! Common functions used across all UI modules.

use bevy::prelude::*;

/// Lightens a color for hover state.
///
/// # Arguments
///
/// * `color` - The base color to lighten
///
/// # Returns
///
/// A new color with increased lightness
pub fn item_hovered(color: Color) -> Color {
    let hsla = Hsla::from(color);
    Color::hsla(
        hsla.hue,
        hsla.saturation,
        (hsla.lightness + 0.1).min(1.0),
        hsla.alpha,
    )
}

/// Lightens a color for pressed state.
///
/// # Arguments
///
/// * `color` - The base color to lighten
///
/// # Returns
///
/// A new color with increased lightness
pub fn item_pressed(color: Color) -> Color {
    let hsla = Hsla::from(color);
    Color::hsla(
        hsla.hue,
        hsla.saturation,
        (hsla.lightness + 0.2).min(1.0),
        hsla.alpha,
    )
}
