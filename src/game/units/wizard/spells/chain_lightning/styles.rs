use bevy::prelude::*;

use super::constants::*;
use crate::game::units::constants::EXCREMAGE_BROWN;

/// Returns the arc width scaled by split depth and empowerment.
/// Deeper splits produce thinner arcs.
pub fn arc_width_at_depth(depth: u32, empowerment: f32) -> f32 {
    let width = ARC_WIDTH * empowerment * ARC_WIDTH_FALLOFF.powi(depth as i32);
    width.max(MIN_ARC_WIDTH)
}

/// Returns the arc color scaled by split depth.
/// Deeper splits produce dimmer arcs. Excremage uses brown tones.
pub fn arc_color_at_depth(depth: u32, is_excremage: bool) -> Color {
    let brightness = ARC_BRIGHTNESS_FALLOFF
        .powi(depth as i32)
        .max(MIN_ARC_BRIGHTNESS);
    let base = if is_excremage {
        EXCREMAGE_BROWN.to_srgba()
    } else {
        ARC_COLOR.to_srgba()
    };
    Color::srgb(
        base.red * brightness,
        base.green * brightness,
        base.blue * brightness,
    )
}
