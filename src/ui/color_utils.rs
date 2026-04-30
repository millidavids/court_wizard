//! Shared UI styling helpers.
//!
//! Common functions used across all UI modules.

use bevy::prelude::*;

/// Purple accent hue used when the source color is neutral/desaturated.
const ACCENT_HUE: f32 = 270.0;

/// Returns the hue to use: the original if saturated, or purple if neutral.
/// Prevents desaturated colors from producing warm/red tones when brightened.
fn accent_hue(hsla: Hsla) -> f32 {
    if hsla.saturation < 0.10 {
        ACCENT_HUE
    } else {
        hsla.hue
    }
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

/// Blends a tint color over a base color using straight-alpha compositing in
/// linear RGB. Blending in HSL produces nonsense hues when the base is near-grey
/// (low saturation → arbitrary stored hue), so RGB is the correct space.
pub fn blend_over(base: Color, tint: Color) -> Color {
    let b = LinearRgba::from(base);
    let t = LinearRgba::from(tint);
    let a = t.alpha;
    Color::LinearRgba(LinearRgba {
        red: b.red * (1.0 - a) + t.red * a,
        green: b.green * (1.0 - a) + t.green * a,
        blue: b.blue * (1.0 - a) + t.blue * a,
        alpha: b.alpha,
    })
}
