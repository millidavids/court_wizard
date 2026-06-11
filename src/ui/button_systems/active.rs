use bevy::prelude::*;
use bevy::ui::ShadowStyle;

use super::super::color_utils::border_bright;
use super::super::components::{
    ButtonActive, ButtonAnimState, ButtonColors, ButtonEdge, ButtonFront,
};
use super::super::constants::{
    BUTTON_3D_OFFSET_PRESSED, BUTTON_3D_OFFSET_REST, BUTTON_PRESS_GLOW_INNER,
    BUTTON_PRESS_GLOW_OUTER, BUTTON_PRESSED_OUTLINE, BUTTON_REST_OUTLINE, BUTTON_SHADOW_COLOR,
};

/// Sets the pressed visual state on newly activated buttons.
/// Only runs when `ButtonActive` is first added, not every frame.
pub fn enforce_active_button_state(
    mut active_buttons: Query<
        (
            &ButtonColors,
            Option<&Children>,
            Option<&mut ButtonAnimState>,
            Option<&mut BoxShadow>,
        ),
        (Added<ButtonActive>, With<Button>),
    >,
    mut front_query: Query<&mut BorderColor, (With<ButtonFront>, Without<ButtonEdge>)>,
    mut edge_query: Query<&mut Outline, With<ButtonEdge>>,
) {
    for (colors, children, anim, shadow) in &mut active_buttons {
        if let Some(mut anim) = anim {
            anim.target = BUTTON_3D_OFFSET_PRESSED;
        }
        if let Some(mut shadow) = shadow {
            shadow.0 = vec![
                ShadowStyle {
                    color: BUTTON_PRESS_GLOW_INNER,
                    x_offset: Val::Px(0.0),
                    y_offset: Val::Px(0.0),
                    spread_radius: Val::Px(4.0),
                    blur_radius: Val::Px(12.0),
                },
                ShadowStyle {
                    color: BUTTON_PRESS_GLOW_OUTER,
                    x_offset: Val::Px(0.0),
                    y_offset: Val::Px(0.0),
                    spread_radius: Val::Px(8.0),
                    blur_radius: Val::Px(24.0),
                },
            ];
        }
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(mut bc) = front_query.get_mut(child) {
                    *bc = BorderColor::all(border_bright(colors.border));
                }
                if let Ok(mut outline) = edge_query.get_mut(child) {
                    outline.color = BUTTON_PRESSED_OUTLINE;
                }
            }
        }
    }
}

/// Resets buttons to their resting state when `ButtonActive` is removed.
pub fn reset_deactivated_buttons(
    mut removed: RemovedComponents<ButtonActive>,
    mut buttons: Query<
        (
            &ButtonColors,
            Option<&Children>,
            Option<&mut ButtonAnimState>,
            Option<&mut BoxShadow>,
        ),
        With<Button>,
    >,
    mut front_query: Query<&mut BorderColor, (With<ButtonFront>, Without<ButtonEdge>)>,
    mut edge_query: Query<&mut Outline, With<ButtonEdge>>,
) {
    for entity in removed.read() {
        let Ok((colors, children, anim, shadow)) = buttons.get_mut(entity) else {
            continue;
        };
        if let Some(mut anim) = anim {
            anim.target = BUTTON_3D_OFFSET_REST;
        }
        if let Some(mut shadow) = shadow {
            shadow.0 = vec![ShadowStyle {
                color: BUTTON_SHADOW_COLOR,
                x_offset: Val::Px(0.0),
                y_offset: Val::Px(2.0),
                spread_radius: Val::Px(0.0),
                blur_radius: Val::Px(4.0),
            }];
        }
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(mut bc) = front_query.get_mut(child) {
                    *bc = BorderColor::all(colors.border);
                }
                if let Ok(mut outline) = edge_query.get_mut(child) {
                    outline.color = BUTTON_REST_OUTLINE;
                }
            }
        }
    }
}
