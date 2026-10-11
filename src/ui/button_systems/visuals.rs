use bevy::prelude::*;
use bevy::ui::ShadowStyle;

use super::super::color_utils::{border_bright, border_hovered};
use super::super::components::{
    ButtonActive, ButtonAnimState, ButtonColors, ButtonEdge, ButtonFront, UiButton,
};
use super::super::constants::{
    BUTTON_3D_OFFSET_HOVER, BUTTON_3D_OFFSET_PRESSED, BUTTON_3D_OFFSET_REST, BUTTON_GLOW_INNER,
    BUTTON_GLOW_OUTER, BUTTON_HOVERED_OUTLINE, BUTTON_PRESS_GLOW_INNER, BUTTON_PRESS_GLOW_OUTER,
    BUTTON_PRESSED_OUTLINE, BUTTON_REST_OUTLINE, BUTTON_SHADOW_COLOR,
};
use super::pointer_visual::{PointerState, PointerVisual, PointerVisualChanges};

/// Sets the 3D button animation target from hover/press/focus state.
///
/// Glows BOTH the edge's outline (lower layer) and the front face's border (top layer).
#[allow(clippy::type_complexity)]
pub(crate) fn update_button_visuals(
    mut changes: PointerVisualChanges,
    mut buttons: Query<
        (
            PointerState,
            &ButtonColors,
            Option<&Children>,
            Option<&mut BoxShadow>,
            Option<&mut ButtonAnimState>,
            Has<ButtonActive>,
        ),
        With<UiButton>,
    >,
    mut front_query: Query<&mut BorderColor, (With<ButtonFront>, Without<ButtonEdge>)>,
    mut edge_query: Query<&mut Outline, With<ButtonEdge>>,
) {
    for entity in changes.entities() {
        let Ok((state, colors, children, shadow, anim, is_active)) = buttons.get_mut(entity) else {
            continue;
        };
        // Active buttons stay in permanent pressed state.
        if is_active {
            continue;
        }
        let visual = PointerVisual::of(state);
        // Pressed is brighter than hover for a satisfying "flash" on click.
        let (front_border, edge_outline) = match visual {
            PointerVisual::Press => (border_bright(colors.border), BUTTON_PRESSED_OUTLINE),
            PointerVisual::Hover => (border_hovered(colors.border), BUTTON_HOVERED_OUTLINE),
            PointerVisual::Rest => (colors.border, BUTTON_REST_OUTLINE),
        };

        // Update edge outline (lower layer glow) + front border (top layer glow).
        // Front-face background tint is handled separately by `apply_gamepad_focus_tint`
        // — mouse hover/press do not tint the bg; only controller focus does.
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(mut bc) = front_query.get_mut(child) {
                    *bc = BorderColor::all(front_border);
                }
                if let Ok(mut outline) = edge_query.get_mut(child) {
                    outline.color = edge_outline;
                }
            }
        }

        // Update wrapper shadow + animation target.
        let (glow, offset) = match visual {
            PointerVisual::Press => (
                vec![
                    glow_shadow(BUTTON_PRESS_GLOW_INNER, 3.0, 10.0),
                    glow_shadow(BUTTON_PRESS_GLOW_OUTER, 6.0, 20.0),
                ],
                BUTTON_3D_OFFSET_PRESSED,
            ),
            PointerVisual::Hover => (
                vec![
                    glow_shadow(BUTTON_GLOW_INNER, 4.0, 12.0),
                    glow_shadow(BUTTON_GLOW_OUTER, 8.0, 24.0),
                ],
                BUTTON_3D_OFFSET_HOVER,
            ),
            PointerVisual::Rest => (
                vec![ShadowStyle {
                    color: BUTTON_SHADOW_COLOR,
                    x_offset: Val::Px(0.0),
                    y_offset: Val::Px(2.0),
                    spread_radius: Val::Px(0.0),
                    blur_radius: Val::Px(4.0),
                }],
                BUTTON_3D_OFFSET_REST,
            ),
        };
        if let Some(mut shadow) = shadow {
            shadow.0 = glow;
        }
        if let Some(mut anim) = anim {
            anim.target = offset;
        }
    }
}

fn glow_shadow(color: Color, spread: f32, blur: f32) -> ShadowStyle {
    ShadowStyle {
        color,
        x_offset: Val::Px(0.0),
        y_offset: Val::Px(0.0),
        spread_radius: Val::Px(spread),
        blur_radius: Val::Px(blur),
    }
}
