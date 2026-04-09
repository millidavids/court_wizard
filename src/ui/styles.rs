//! Shared UI styling helpers.
//!
//! Common functions used across all UI modules.

use bevy::prelude::*;

/// Purple accent hue used when the source color is neutral/desaturated.
const ACCENT_HUE: f32 = 270.0;

/// Returns the hue to use: the original if saturated, or purple if neutral.
/// Prevents desaturated colors from producing warm/red tones when brightened.
fn accent_hue(hsla: Hsla) -> f32 {
    if hsla.saturation < 0.10 { ACCENT_HUE } else { hsla.hue }
}

/// Brightens border color for hover state (+15% lightness, +15% saturation).
pub fn border_hovered(color: Color) -> Color {
    let hsla = Hsla::from(color);
    Color::hsla(
        accent_hue(hsla),
        (hsla.saturation + 0.15).min(1.0),
        (hsla.lightness + 0.15).min(1.0),
        hsla.alpha,
    )
}

/// Brightens border color for pressed state — brighter than hover for a flash effect.
pub fn border_bright(color: Color) -> Color {
    let hsla = Hsla::from(color);
    Color::hsla(
        accent_hue(hsla),
        (hsla.saturation + 0.25).min(1.0),
        (hsla.lightness + 0.25).min(1.0),
        1.0,
    )
}
