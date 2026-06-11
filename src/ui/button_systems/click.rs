use bevy::prelude::*;
use bevy::ui::ShadowStyle;

use super::super::color_utils::{border_bright, border_hovered};
use super::super::components::{
    ButtonActive, ButtonAnimState, ButtonColors, ButtonEdge, ButtonFront,
};
use super::super::constants::{
    BUTTON_3D_OFFSET_HOVER, BUTTON_3D_OFFSET_PRESSED, BUTTON_3D_OFFSET_REST, BUTTON_GLOW_INNER,
    BUTTON_GLOW_OUTER, BUTTON_HOVERED_OUTLINE, BUTTON_PRESS_GLOW_INNER, BUTTON_PRESS_GLOW_OUTER,
    BUTTON_PRESSED_OUTLINE, BUTTON_REST_OUTLINE, BUTTON_SHADOW_COLOR,
};
use crate::game::input::messages::MouseClicked;

/// Marker component to track that a button was pressed down.
#[derive(Component)]
pub struct ButtonPressedDown;

/// Run condition that returns true if there are any MouseClicked messages.
pub fn on_message<M: Message>(mut reader: MessageReader<M>) -> bool {
    reader.read().next().is_some()
}

/// Sets the 3D button animation target based on interaction state.
///
/// Glows BOTH the edge's outline (lower layer) and the front face's border (top layer).
#[allow(clippy::type_complexity)]
pub fn button_interaction(
    mut interaction_query: Query<
        (
            &Interaction,
            &ButtonColors,
            Option<&Children>,
            Option<&mut BoxShadow>,
            Option<&mut ButtonAnimState>,
            Has<ButtonActive>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut front_query: Query<&mut BorderColor, (With<ButtonFront>, Without<ButtonEdge>)>,
    mut edge_query: Query<&mut Outline, With<ButtonEdge>>,
) {
    for (interaction, colors, children, shadow, anim, is_active) in &mut interaction_query {
        // Active buttons stay in permanent pressed state.
        if is_active {
            continue;
        }
        // Determine hover colors for both layers.
        // Pressed is brighter than hover for a satisfying "flash" on click.
        let (front_border, edge_outline) = match *interaction {
            Interaction::Pressed => (border_bright(colors.border), BUTTON_PRESSED_OUTLINE),
            Interaction::Hovered => (border_hovered(colors.border), BUTTON_HOVERED_OUTLINE),
            Interaction::None => (colors.border, BUTTON_REST_OUTLINE),
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
        match *interaction {
            Interaction::Pressed => {
                if let Some(mut shadow) = shadow {
                    shadow.0 = vec![
                        ShadowStyle {
                            color: BUTTON_PRESS_GLOW_INNER,
                            x_offset: Val::Px(0.0),
                            y_offset: Val::Px(0.0),
                            spread_radius: Val::Px(3.0),
                            blur_radius: Val::Px(10.0),
                        },
                        ShadowStyle {
                            color: BUTTON_PRESS_GLOW_OUTER,
                            x_offset: Val::Px(0.0),
                            y_offset: Val::Px(0.0),
                            spread_radius: Val::Px(6.0),
                            blur_radius: Val::Px(20.0),
                        },
                    ];
                }
                if let Some(mut anim) = anim {
                    anim.target = BUTTON_3D_OFFSET_PRESSED;
                }
            }
            Interaction::Hovered => {
                if let Some(mut shadow) = shadow {
                    shadow.0 = vec![
                        ShadowStyle {
                            color: BUTTON_GLOW_INNER,
                            x_offset: Val::Px(0.0),
                            y_offset: Val::Px(0.0),
                            spread_radius: Val::Px(4.0),
                            blur_radius: Val::Px(12.0),
                        },
                        ShadowStyle {
                            color: BUTTON_GLOW_OUTER,
                            x_offset: Val::Px(0.0),
                            y_offset: Val::Px(0.0),
                            spread_radius: Val::Px(8.0),
                            blur_radius: Val::Px(24.0),
                        },
                    ];
                }
                if let Some(mut anim) = anim {
                    anim.target = BUTTON_3D_OFFSET_HOVER;
                }
            }
            Interaction::None => {
                if let Some(mut shadow) = shadow {
                    shadow.0 = vec![ShadowStyle {
                        color: BUTTON_SHADOW_COLOR,
                        x_offset: Val::Px(0.0),
                        y_offset: Val::Px(2.0),
                        spread_radius: Val::Px(0.0),
                        blur_radius: Val::Px(4.0),
                    }];
                }
                if let Some(mut anim) = anim {
                    anim.target = BUTTON_3D_OFFSET_REST;
                }
            }
        }
    }
}

/// Tracks button press state and sends click events.
///
/// This system handles the core button click detection:
/// - Marks buttons as pressed when interaction becomes Pressed
/// - Sends MouseClicked event when interaction changes from Pressed to non-Pressed (either Hovered or None)
/// - Only sends click event if the button was previously marked as pressed down
///
/// This works for both mouse (Pressed → Hovered → None) and touch (Pressed → None).
pub fn button_click_detection(
    mut commands: Commands,
    mut interaction_query: Query<
        (Entity, &Interaction, Option<&ButtonPressedDown>),
        (Changed<Interaction>, With<Button>),
    >,
    mut button_clicked: MessageWriter<MouseClicked>,
) {
    for (entity, interaction, pressed_down) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Mark button as pressed down
                commands.entity(entity).insert(ButtonPressedDown);
            }
            Interaction::Hovered | Interaction::None => {
                // If button was pressed down and is now released, send click event
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();
                    button_clicked.write(MouseClicked { button: entity });
                }
            }
        }
    }
}
