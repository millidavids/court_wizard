//! UI button systems: interaction, 3D structure, gamepad focus, color sync.

use bevy::prelude::*;
use bevy::ui::ShadowStyle;

use super::color_utils::{blend_over, border_bright, border_hovered};
use super::components::{ButtonActive, ButtonAnimState, ButtonColors, ButtonEdge, ButtonFront};
use super::constants::{
    BUTTON_3D_ANIM_SPEED, BUTTON_3D_OFFSET_HOVER, BUTTON_3D_OFFSET_PRESSED, BUTTON_3D_OFFSET_REST,
    BUTTON_EDGE_DARKEN, BUTTON_GLOW_INNER, BUTTON_GLOW_OUTER, BUTTON_HOVER_BG_TINT,
    BUTTON_HOVERED_OUTLINE, BUTTON_PRESS_GLOW_INNER, BUTTON_PRESS_GLOW_OUTER,
    BUTTON_PRESSED_OUTLINE, BUTTON_REST_OUTLINE, BUTTON_SHADOW_COLOR,
};
use super::focus::GamepadFocused;
use crate::game::input::messages::MouseClicked;

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

/// Marker component to track that a button was pressed down.
#[derive(Component)]
pub struct ButtonPressedDown;

/// Marker for page content panels that should receive a parchment background.
#[derive(Component)]
pub(crate) struct ParchmentPanel;

/// Marker for overlay roots that should receive a frosted glass background.
#[derive(Component)]
pub(crate) struct FrostedGlassOverlay;

/// Run condition that returns true if there are any MouseClicked messages.
pub fn on_message<M: Message>(mut reader: MessageReader<M>) -> bool {
    reader.read().next().is_some()
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

/// Converts any flat button (with `ButtonColors` but no `ButtonAnimState`) into
/// a 3D layered button. Reparents existing children into a front face child and
/// adds an edge child behind it.
///
/// The front face inherits the original button's layout properties (flex direction,
/// alignment, padding, gaps) so content renders identically.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn apply_3d_button_structure(
    mut commands: Commands,
    new_buttons: Query<
        (
            Entity,
            &ButtonColors,
            &Node,
            Option<&BorderColor>,
            Option<&Children>,
            Has<ButtonActive>,
        ),
        (Added<ButtonColors>, With<Button>, Without<ButtonAnimState>),
    >,
) {
    let depth = -BUTTON_3D_OFFSET_REST;

    for (entity, colors, node, border_color, children, is_active) in &new_buttons {
        // Skip transparent/utility buttons — the 3D effect doesn't suit them
        // and restructuring breaks click behavior for small nested buttons.
        let bg_hsla = Hsla::from(colors.background);
        if bg_hsla.alpha < 0.01 {
            continue;
        }

        let br = if node.border_radius == BorderRadius::ZERO {
            BorderRadius::all(Val::Px(4.0))
        } else {
            node.border_radius
        };
        let bc = border_color
            .copied()
            .unwrap_or(BorderColor::all(colors.border));
        let border_width = match node.border.top {
            Val::Px(px) => px,
            _ => 1.0,
        };
        let original_height = match node.height {
            Val::Px(h) => h,
            _ => 40.0,
        };

        // Collect existing children to reparent into the front face.
        let existing_children: Vec<Entity> =
            children.map(|c| c.iter().collect()).unwrap_or_default();

        // If the button is already active, start in pressed state
        let (initial_offset, front_border, edge_outline_color) = if is_active {
            (
                BUTTON_3D_OFFSET_PRESSED,
                BorderColor::all(border_bright(colors.border)),
                BUTTON_PRESSED_OUTLINE,
            )
        } else {
            (BUTTON_3D_OFFSET_REST, bc, BUTTON_REST_OUTLINE)
        };

        // Spawn edge — same size as front but offset down and slightly narrower.
        // Outline lives here so it appears to go behind the front face.
        // Active buttons get the pressed outline immediately.
        let edge = commands
            .spawn((
                ButtonEdge,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    border_radius: br,
                    ..default()
                },
                BackgroundColor(edge_color(colors.background)),
                Outline::new(Val::Px(1.0), Val::Px(1.0), edge_outline_color),
            ))
            .id();

        // Front face inherits layout from original button so content stays correct.
        // Fixed-height buttons stay fixed; auto-sized buttons can grow.
        let has_fixed_height = matches!(node.height, Val::Px(_));
        let mut front_node = Node {
            width: Val::Percent(100.0),
            border: UiRect::all(Val::Px(border_width)),
            flex_direction: node.flex_direction,
            justify_content: node.justify_content,
            align_items: node.align_items,
            padding: node.padding,
            row_gap: node.row_gap,
            column_gap: node.column_gap,
            flex_wrap: node.flex_wrap,
            overflow: node.overflow,
            position_type: PositionType::Relative,
            top: Val::Px(initial_offset),
            border_radius: br,
            ..default()
        };
        if has_fixed_height {
            front_node.height = Val::Px(original_height);
            front_node.overflow = Overflow::clip();
        } else {
            front_node.min_height = Val::Px(original_height);
        }

        let front = commands
            .spawn((
                ButtonFront,
                front_node,
                BackgroundColor(opaque(colors.background)),
                front_border,
            ))
            .id();

        // Reparent existing children (text, icons) into the front face.
        for child in &existing_children {
            commands.entity(front).add_child(*child);
        }

        // Update the wrapper: clear bg/border, increase height, add 3D components.
        // Outline stays on wrapper (lower layer) — glows on hover.
        commands.entity(entity).insert((
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            ButtonAnimState {
                current: initial_offset,
                target: initial_offset,
            },
            BoxShadow(if is_active {
                vec![
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
                ]
            } else {
                vec![ShadowStyle {
                    color: BUTTON_SHADOW_COLOR,
                    x_offset: Val::Px(0.0),
                    y_offset: Val::Px(2.0),
                    spread_radius: Val::Px(0.0),
                    blur_radius: Val::Px(4.0),
                }]
            }),
        ));

        // Increase wrapper height, clear border/padding (those live on front now).
        commands
            .entity(entity)
            .entry::<Node>()
            .and_modify(move |mut n| {
                // Convert fixed height to min_height so content can grow.
                if let Val::Px(h) = n.height {
                    n.min_height = Val::Px(h + depth);
                    n.height = Val::Auto;
                } else {
                    // Already auto-sized; just add depth padding at bottom.
                    n.padding.bottom = Val::Px(depth);
                }
                n.border = UiRect::ZERO;
                n.padding.top = Val::ZERO;
                n.padding.left = Val::ZERO;
                n.padding.right = Val::ZERO;
                n.row_gap = Val::ZERO;
                n.column_gap = Val::ZERO;
            });

        // Add edge + front as children (after existing children were reparented).
        commands.entity(entity).add_child(edge);
        commands.entity(entity).add_child(front);
    }
}

/// Smoothly animates the 3D button front face toward its target offset.
/// Uses real (wall-clock) time so animations play even when game time is paused/scaled.
/// Updates both the Node.top on the front face and ButtonAnimState.current in one pass.
pub fn animate_button_3d(
    time: Res<Time<Real>>,
    mut buttons: Query<(&mut ButtonAnimState, &Children)>,
    mut front_query: Query<&mut Node, With<ButtonFront>>,
) {
    let dt = time.delta_secs();
    for (mut anim, children) in &mut buttons {
        if (anim.current - anim.target).abs() < 0.01 {
            anim.current = anim.target;
            continue;
        }

        let speed = if anim.target == BUTTON_3D_OFFSET_PRESSED {
            BUTTON_3D_ANIM_SPEED * 3.0
        } else {
            BUTTON_3D_ANIM_SPEED
        };
        let t = (speed * dt).min(1.0);
        anim.current += (anim.target - anim.current) * t;

        for child in children.iter() {
            if let Ok(mut node) = front_query.get_mut(child) {
                node.top = Val::Px(anim.current);
            }
        }
    }
}

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

/// Tints the 3D front-face background purple when a button is gamepad-focused,
/// and restores the base (charcoal) color when focus leaves. This is the only
/// visual that distinguishes controller focus from mouse hover — mouse hover
/// keeps the default bg and only tweaks the border / outline / glow.
#[allow(clippy::type_complexity)]
pub fn apply_gamepad_focus_tint(
    focused: Query<
        (&ButtonColors, &Children),
        Or<(
            Added<GamepadFocused>,
            (With<GamepadFocused>, Changed<ButtonColors>),
        )>,
    >,
    mut removed: RemovedComponents<GamepadFocused>,
    all_buttons: Query<(&ButtonColors, &Children)>,
    mut front_query: Query<&mut BackgroundColor, (With<ButtonFront>, Without<ButtonEdge>)>,
) {
    for (colors, children) in &focused {
        let tinted = blend_over(opaque(colors.background), BUTTON_HOVER_BG_TINT);
        for child in children.iter() {
            if let Ok(mut bg) = front_query.get_mut(child) {
                *bg = BackgroundColor(tinted);
            }
        }
    }

    for entity in removed.read() {
        if let Ok((colors, children)) = all_buttons.get(entity) {
            let base = opaque(colors.background);
            for child in children.iter() {
                if let Ok(mut bg) = front_query.get_mut(child) {
                    *bg = BackgroundColor(base);
                }
            }
        }
    }
}

/// Tints the entity's own `BackgroundColor` purple when gamepad-focused, for
/// flat focusables that don't wrap a `ButtonFront` child (e.g. text input
/// fields). Stores the base color in `FocusableFlatBackground` so the tint
/// can be cleanly removed on unfocus.
pub fn apply_flat_gamepad_focus_tint(
    mut just_focused: Query<
        (&super::focus::FocusableFlatBackground, &mut BackgroundColor),
        Added<GamepadFocused>,
    >,
    mut removed: RemovedComponents<GamepadFocused>,
    mut un_focused: Query<
        (&super::focus::FocusableFlatBackground, &mut BackgroundColor),
        Without<GamepadFocused>,
    >,
) {
    for (tint, mut bg) in &mut just_focused {
        *bg = BackgroundColor(blend_over(opaque(tint.base), BUTTON_HOVER_BG_TINT));
    }
    for entity in removed.read() {
        if let Ok((tint, mut bg)) = un_focused.get_mut(entity) {
            *bg = BackgroundColor(tint.base);
        }
    }
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

/// Inserts an absolutely positioned `MaterialNode` child behind existing content.
///
/// Clears the parent's `BackgroundColor` so the shader shows through.
/// The shaders themselves handle rounded-corner clipping via `in.border_radius`.
pub(super) fn insert_material_background<M: UiMaterial>(
    commands: &mut Commands,
    entity: Entity,
    material: Handle<M>,
) {
    let child = commands
        .spawn((
            MaterialNode(material),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
        ))
        .id();

    commands.entity(entity).insert(BackgroundColor(Color::NONE));
    commands.entity(entity).insert_children(0, &[child]);
}

/// Derives a semi-transparent edge (depth) color from a button's background color.
pub(super) fn edge_color(bg: Color) -> Color {
    let hsla = Hsla::from(bg);
    Color::hsla(
        hsla.hue,
        (hsla.saturation + 0.05).min(1.0),
        hsla.lightness * BUTTON_EDGE_DARKEN,
        0.8,
    )
}

/// Returns a fully opaque version of a color for the front face.
pub(crate) fn opaque(color: Color) -> Color {
    let hsla = Hsla::from(color);
    Color::hsla(hsla.hue, hsla.saturation, hsla.lightness, 1.0)
}
