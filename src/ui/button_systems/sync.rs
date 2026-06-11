use bevy::prelude::*;

use super::super::components::{ButtonActive, ButtonColors, ButtonEdge, ButtonFront};
use super::super::constants::BUTTON_REST_OUTLINE;

/// Scales a font size down based on text width to fit within a constrained area.
///
/// Returns `base_font` when `max_width <= min_chars`, scaling linearly down to
/// `base_font * min_scale` when `max_width >= max_chars`.
pub(crate) fn scale_font_by_text_width(
    max_width: f32,
    min_chars: f32,
    max_chars: f32,
    min_scale: f32,
    base_font: f32,
) -> f32 {
    let t = ((max_width - min_chars) / (max_chars - min_chars)).clamp(0.0, 1.0);
    base_font * (1.0 - t * (1.0 - min_scale))
}

/// Returns a fully opaque version of a color for the front face.
pub(crate) fn opaque(color: Color) -> Color {
    let hsla = Hsla::from(color);
    Color::hsla(hsla.hue, hsla.saturation, hsla.lightness, 1.0)
}

/// Syncs the front face's `BorderColor` and `BackgroundColor` when the wrapper's
/// `ButtonColors` changes. This ensures that when external systems update button
/// colors (e.g., selecting a wizard card, toggling a modifier), the 3D front face
/// reflects the change.
pub fn sync_front_face_colors(
    changed_buttons: Query<
        (&ButtonColors, &Children),
        (Changed<ButtonColors>, With<Button>, Without<ButtonActive>),
    >,
    mut front_query: Query<
        (&mut BackgroundColor, &mut BorderColor),
        (With<ButtonFront>, Without<ButtonEdge>),
    >,
    mut edge_query: Query<&mut Outline, With<ButtonEdge>>,
) {
    for (colors, children) in &changed_buttons {
        for child in children.iter() {
            if let Ok((mut bg, mut border)) = front_query.get_mut(child) {
                *bg = opaque(colors.background).into();
                *border = BorderColor::all(colors.border);
            }
            if let Ok(mut outline) = edge_query.get_mut(child) {
                outline.color = BUTTON_REST_OUTLINE;
            }
        }
    }
}
