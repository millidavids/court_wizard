use bevy::prelude::*;
use bevy::ui::ShadowStyle;

use super::super::color_utils::border_bright;
use super::super::components::{
    ButtonActive, ButtonAnimState, ButtonColors, ButtonEdge, ButtonFront,
};
use super::super::constants::{
    BUTTON_3D_OFFSET_PRESSED, BUTTON_3D_OFFSET_REST, BUTTON_EDGE_DARKEN, BUTTON_PRESS_GLOW_INNER,
    BUTTON_PRESS_GLOW_OUTER, BUTTON_PRESSED_OUTLINE, BUTTON_REST_OUTLINE, BUTTON_SHADOW_COLOR,
};
use super::sync::opaque;

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

/// Inserts an absolutely positioned `MaterialNode` child behind existing content.
///
/// Clears the parent's `BackgroundColor` so the shader shows through.
/// The shaders themselves handle rounded-corner clipping via `in.border_radius`.
pub(crate) fn insert_material_background<M: UiMaterial>(
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
pub(crate) fn edge_color(bg: Color) -> Color {
    let hsla = Hsla::from(bg);
    Color::hsla(
        hsla.hue,
        (hsla.saturation + 0.05).min(1.0),
        hsla.lightness * BUTTON_EDGE_DARKEN,
        0.8,
    )
}
