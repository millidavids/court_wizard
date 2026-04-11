//! Shared UI systems used across all menus and screens.

use bevy::ecs::relationship::Relationship;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, ShadowStyle};

use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::ui::RelativeCursorPosition;

use super::components::{ButtonActive, ButtonAnimState, ButtonColors, ButtonEdge, ButtonFront, ButtonStyle};
use super::constants::{
    BUTTON_3D_ANIM_SPEED, BUTTON_3D_OFFSET_HOVER, BUTTON_3D_OFFSET_PRESSED, BUTTON_3D_OFFSET_REST,
    BUTTON_EDGE_DARKEN, BUTTON_GLOW_INNER, BUTTON_GLOW_OUTER, BUTTON_HOVERED_OUTLINE,
    BUTTON_PRESSED_OUTLINE, BUTTON_PRESS_GLOW_INNER, BUTTON_PRESS_GLOW_OUTER,
    BUTTON_SHADOW_COLOR, CONTENT_BG, CONTENT_BORDER, DETAIL_BG,
    DETAIL_BORDER, DETAIL_PADDING, FRAME_OUTER_RING_COLOR, FRAME_OUTLINE_COLOR,
    FRAME_OUTLINE_OFFSET, FRAME_OUTLINE_WIDTH, FRAME_SHADOW_SPREAD_BASE, LEFT_PANEL_WIDTH, LIST_BG,
    LIST_BORDER, OVERLAY_BG, PANEL_BORDER_RADIUS, SCROLL_BG, SCROLL_BORDER, SCROLL_SHADOW_COLOR,
    SHADOW_COLOR, SLIDER_BORDER_WIDTH, SLIDER_BUTTON_BG, SLIDER_BUTTON_BORDER_COLOR,
    SLIDER_BUTTON_FONT_SIZE, SLIDER_BUTTON_SIZE, SLIDER_GAP, SLIDER_LABEL_FONT_SIZE,
    SLIDER_TRACK_WIDTH, TEXT_PRIMARY, TEXT_SHADOW_COLOR,
};
use super::styles::{border_bright, border_hovered};
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseClicked;
use crate::state::{InGameState, MenuState, MultiplayerGameState, PauseMenuState};

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
            Interaction::None => (colors.border, FRAME_OUTLINE_COLOR),
        };

        // Update edge outline (lower layer glow) + front border (top layer glow).
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
                            spread_radius: Val::Px(2.0),
                            blur_radius: Val::Px(8.0),
                        },
                        ShadowStyle {
                            color: BUTTON_GLOW_OUTER,
                            x_offset: Val::Px(0.0),
                            y_offset: Val::Px(0.0),
                            spread_radius: Val::Px(4.0),
                            blur_radius: Val::Px(16.0),
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
            Option<&BorderRadius>,
            Option<&BorderColor>,
            Option<&Children>,
            Has<ButtonActive>,
        ),
        (Added<ButtonColors>, With<Button>, Without<ButtonAnimState>),
    >,
) {
    let depth = -BUTTON_3D_OFFSET_REST;

    for (entity, colors, node, radius, border_color, children, is_active) in &new_buttons {
        // Skip transparent/utility buttons — the 3D effect doesn't suit them
        // and restructuring breaks click behavior for small nested buttons.
        let bg_hsla = Hsla::from(colors.background);
        if bg_hsla.alpha < 0.01 {
            continue;
        }

        let br = radius.copied().unwrap_or(BorderRadius::all(Val::Px(4.0)));
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
            (BUTTON_3D_OFFSET_REST, bc, FRAME_OUTLINE_COLOR)
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
                    ..default()
                },
                BackgroundColor(edge_color(colors.background)),
                br,
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
                br,
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
            BoxShadow(vec![ShadowStyle {
                color: BUTTON_SHADOW_COLOR,
                x_offset: Val::Px(0.0),
                y_offset: Val::Px(2.0),
                spread_radius: Val::Px(0.0),
                blur_radius: Val::Px(4.0),
            }]),
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
        (&ButtonColors, Option<&Children>, Option<&mut ButtonAnimState>),
        (Added<ButtonActive>, With<Button>),
    >,
    mut front_query: Query<&mut BorderColor, (With<ButtonFront>, Without<ButtonEdge>)>,
    mut edge_query: Query<&mut Outline, With<ButtonEdge>>,
) {
    for (colors, children, anim) in &mut active_buttons {
        if let Some(mut anim) = anim {
            anim.target = BUTTON_3D_OFFSET_PRESSED;
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
        (&ButtonColors, Option<&Children>, Option<&mut ButtonAnimState>),
        With<Button>,
    >,
    mut front_query: Query<&mut BorderColor, (With<ButtonFront>, Without<ButtonEdge>)>,
    mut edge_query: Query<&mut Outline, With<ButtonEdge>>,
) {
    for entity in removed.read() {
        let Ok((colors, children, anim)) = buttons.get_mut(entity) else {
            continue;
        };
        if let Some(mut anim) = anim {
            anim.target = BUTTON_3D_OFFSET_REST;
        }
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(mut bc) = front_query.get_mut(child) {
                    *bc = BorderColor::all(colors.border);
                }
                if let Ok(mut outline) = edge_query.get_mut(child) {
                    outline.color = FRAME_OUTLINE_COLOR;
                }
            }
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
                outline.color = FRAME_OUTLINE_COLOR;
            }
        }
    }
}

/// Inserts an absolutely positioned `MaterialNode` child behind existing content.
///
/// Clears the parent's `BackgroundColor` so the shader shows through.
/// The shaders themselves handle rounded-corner clipping via `in.border_radius`.
fn insert_material_background<M: UiMaterial>(
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
fn edge_color(bg: Color) -> Color {
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

/// Applies a parchment texture material to newly spawned panels marked with `ParchmentPanel`.
///
/// Inserts an absolutely positioned `MaterialNode` child behind the panel content
/// to give a weathered, medieval parchment background.
pub fn apply_parchment_backgrounds(
    mut commands: Commands,
    new_panels: Query<Entity, Added<ParchmentPanel>>,
    mut materials: ResMut<Assets<ParchmentMaterial>>,
) {
    for entity in &new_panels {
        let mat = materials.add(ParchmentMaterial::new(CONTENT_BG));
        insert_material_background(&mut commands, entity, mat);
    }
}

/// Applies a frosted glass material to newly spawned overlay roots.
pub fn apply_frosted_glass_overlays(
    mut commands: Commands,
    new_overlays: Query<Entity, Added<FrostedGlassOverlay>>,
    mut materials: ResMut<Assets<FrostedGlassMaterial>>,
) {
    for entity in &new_overlays {
        let mat = materials.add(FrostedGlassMaterial::new());
        insert_material_background(&mut commands, entity, mat);
    }
}

// ---------------------------------------------------------------------------
// Page container (shared by settings, progress, instructions, and overlays)
// ---------------------------------------------------------------------------

/// Returns the shared styling bundle for inner scrollable areas.
/// Callers should also add `ScrollPosition::default()` and their marker component.
pub(crate) fn scroll_area_style() -> (BackgroundColor, BorderColor, BorderRadius, BoxShadow) {
    (
        BackgroundColor(SCROLL_BG),
        BorderColor::all(SCROLL_BORDER),
        BorderRadius::all(Val::Px(4.0)),
        BoxShadow(vec![
            ShadowStyle {
                color: SCROLL_SHADOW_COLOR,
                x_offset: Val::Px(0.0),
                y_offset: Val::Px(1.0),
                spread_radius: Val::Px(0.0),
                blur_radius: Val::Px(2.0),
            },
            ShadowStyle {
                color: SHADOW_COLOR,
                x_offset: Val::Px(0.0),
                y_offset: Val::Px(3.0),
                spread_radius: Val::Px(1.0),
                blur_radius: Val::Px(6.0),
            },
        ]),
    )
}

/// Standard content node for page containers (column, centered, with scroll clipping).
pub fn default_content_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(20.0)),
        border: UiRect::all(Val::Px(1.0)),
        overflow: Overflow::clip(),
        ..default()
    }
}

/// Spawns the standard left detail panel (300px fixed, gold-bordered box).
/// Returns the **inner detail box** entity — add your content as children of this.
pub fn spawn_left_detail_panel(parent: &mut ChildSpawnerCommands) -> Entity {
    let mut detail_box_id = Entity::PLACEHOLDER;
    parent
        .spawn(Node {
            width: Val::Px(LEFT_PANEL_WIDTH),
            flex_direction: FlexDirection::Column,
            align_self: AlignSelf::Center,
            row_gap: Val::Px(16.0),
            flex_grow: 0.0,
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|left| {
            detail_box_id = left
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(DETAIL_PADDING)),
                        row_gap: Val::Px(12.0),
                        border: UiRect::all(Val::Px(1.0)),
                        flex_grow: 1.0,
                        ..default()
                    },
                    BackgroundColor(DETAIL_BG),
                    BorderColor::all(DETAIL_BORDER),
                    BorderRadius::all(Val::Px(PANEL_BORDER_RADIUS)),
                    Outline::new(
                        Val::Px(FRAME_OUTLINE_WIDTH),
                        Val::Px(1.0),
                        FRAME_OUTLINE_COLOR,
                    ),
                ))
                .id();
        });
    detail_box_id
}

/// Like `spawn_left_detail_panel`, but makes the detail box scrollable.
/// Inserts the given `marker` component and `ScrollPosition` for scroll handling.
pub fn spawn_scrollable_left_detail_panel<M: Component>(
    parent: &mut ChildSpawnerCommands,
    marker: M,
) -> Entity {
    let detail_box = spawn_left_detail_panel(parent);
    parent
        .commands()
        .entity(detail_box)
        .insert((marker, ScrollPosition::default()));
    parent
        .commands()
        .entity(detail_box)
        .entry::<Node>()
        .and_modify(|mut node| {
            node.overflow = Overflow::scroll_y();
        });
    detail_box
}

/// Spawns the standard right scrollable panel (flex-grow, dark background with scroll).
/// `marker` is attached for screen-specific queries (e.g. scroll handling).
/// Returns the scrollable content entity — add your content as children of this.
pub fn spawn_right_scroll_panel<M: Component>(
    parent: &mut ChildSpawnerCommands,
    marker: M,
    inner_direction: FlexDirection,
    inner_gap: f32,
) -> Entity {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                min_width: Val::Px(0.0),
                flex_direction: inner_direction,
                row_gap: Val::Px(inner_gap),
                column_gap: Val::Px(inner_gap),
                overflow: Overflow::scroll_y(),
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(LIST_BG),
            BorderColor::all(LIST_BORDER),
            BorderRadius::all(Val::Px(PANEL_BORDER_RADIUS)),
            ScrollPosition::default(),
            marker,
        ))
        .id()
}

/// Spawns a full-screen page with a semi-transparent overlay and a styled
/// content container inside it. Returns the content container entity so the
/// caller can add children.
///
/// `content_node` controls the inner container layout (flex direction, padding,
/// gaps, overflow, etc.). Use `default_content_node()` for the standard look.
/// The border color, background, border-radius, and shadow are applied
/// automatically.
///
/// When `pause_menu` is true, `GlobalZIndex(500)` is added so the page
/// renders above in-game UI.
pub fn spawn_page_container<M: Component>(
    commands: &mut Commands,
    screen_marker: M,
    pause_menu: bool,
    content_node: Node,
) -> Entity {
    let mut root = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(OVERLAY_BG),
        // Make the overlay block all interactions behind it.
        Interaction::default(),
        bevy::ui::FocusPolicy::Block,
        FrostedGlassOverlay,
        screen_marker,
    ));

    if pause_menu {
        root.insert(GlobalZIndex(500));
    }

    let root_id = root.id();

    let content_id = commands
        .spawn((
            content_node,
            BackgroundColor(CONTENT_BG),
            BorderColor::all(CONTENT_BORDER),
            BorderRadius::all(Val::Px(6.0)),
            // Middle ring via outline with gap
            Outline::new(
                Val::Px(FRAME_OUTLINE_WIDTH),
                Val::Px(FRAME_OUTLINE_OFFSET),
                FRAME_OUTLINE_COLOR,
            ),
            BoxShadow(vec![
                // Outermost solid ring (zero blur = solid border)
                ShadowStyle {
                    color: FRAME_OUTER_RING_COLOR,
                    x_offset: Val::Px(0.0),
                    y_offset: Val::Px(0.0),
                    spread_radius: Val::Px(FRAME_SHADOW_SPREAD_BASE),
                    blur_radius: Val::Px(0.0),
                },
                // Tight contact shadow
                ShadowStyle {
                    color: Color::hsla(25.0, 0.20, 0.08, 0.4),
                    x_offset: Val::Px(0.0),
                    y_offset: Val::Px(1.0),
                    spread_radius: Val::Px(FRAME_SHADOW_SPREAD_BASE),
                    blur_radius: Val::Px(3.0),
                },
                // Medium depth shadow
                ShadowStyle {
                    color: Color::hsla(25.0, 0.15, 0.05, 0.3),
                    x_offset: Val::Px(0.0),
                    y_offset: Val::Px(4.0),
                    spread_radius: Val::Px(FRAME_SHADOW_SPREAD_BASE + 2.0),
                    blur_radius: Val::Px(8.0),
                },
                // Wide ambient shadow
                ShadowStyle {
                    color: Color::hsla(25.0, 0.10, 0.03, 0.2),
                    x_offset: Val::Px(0.0),
                    y_offset: Val::Px(8.0),
                    spread_radius: Val::Px(FRAME_SHADOW_SPREAD_BASE + 4.0),
                    blur_radius: Val::Px(20.0),
                },
            ]),
        ))
        .id();

    commands.entity(root_id).add_child(content_id);

    content_id
}

// ---------------------------------------------------------------------------
// Shared Escape key handling
// ---------------------------------------------------------------------------

/// Handles Escape key to return to the main menu landing screen.
pub fn escape_to_landing(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        channel_change.write(ChannelChangeMessage);
        next_state.set(MenuState::Landing);
    }
}

/// Handles Escape key to return to the pause menu main screen.
pub fn escape_to_pause_main(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<PauseMenuState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(PauseMenuState::Main);
    }
}

/// Handles Escape key to return to running gameplay state (SP and/or MP).
pub fn escape_to_running(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_in_game_state: Option<ResMut<NextState<InGameState>>>,
    mut next_mp_state: Option<ResMut<NextState<MultiplayerGameState>>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if let Some(ref mut next_sp) = next_in_game_state {
            next_sp.set(InGameState::Running);
        }
        if let Some(ref mut next_mp) = next_mp_state {
            next_mp.set(MultiplayerGameState::Running);
        }
    }
}

/// Consumes the mouse button state on exit to prevent click bleed-through.
pub fn consume_mouse_on_exit(mut mouse_state: ResMut<MouseButtonState>) {
    mouse_state.left_consumed = true;
}

// ---------------------------------------------------------------------------
// Shared scroll handling
// ---------------------------------------------------------------------------

/// Handles mouse wheel scrolling for any scrollable container marked with `T`.
///
/// Walks up the entity hierarchy from the hovered entity to find a scrollable
/// parent, then adjusts its `ScrollPosition` based on the mouse wheel delta.
pub fn handle_scroll<T: Component>(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    hover_map: Res<bevy::picking::hover::HoverMap>,
    mut scrollable_query: Query<(&mut ScrollPosition, &ComputedNode), With<T>>,
    parent_query: Query<&ChildOf>,
) {
    use bevy::input::mouse::MouseScrollUnit;

    const LINE_HEIGHT: f32 = 10.0;
    const PIXEL_SCROLL_MULTIPLIER: f32 = 0.3;

    'event: for event in mouse_wheel_events.read() {
        let dy = match event.unit {
            MouseScrollUnit::Line => -event.y * LINE_HEIGHT,
            MouseScrollUnit::Pixel => -event.y * PIXEL_SCROLL_MULTIPLIER,
        };

        for pointer_map in hover_map.values() {
            for (hovered_entity, _) in pointer_map.iter() {
                let mut current_entity = *hovered_entity;
                loop {
                    if let Ok((mut scroll_position, computed)) =
                        scrollable_query.get_mut(current_entity)
                    {
                        let visible_size = computed.size();
                        let content_size = computed.content_size();
                        let max_scroll = (content_size.y - visible_size.y).max(0.0)
                            * computed.inverse_scale_factor();

                        scroll_position.y = (scroll_position.y + dy).clamp(0.0, max_scroll);
                        continue 'event;
                    }

                    if let Ok(parent) = parent_query.get(current_entity) {
                        current_entity = parent.get();
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Shared button spawning
// ---------------------------------------------------------------------------

/// Generic screen cleanup system that despawns all entities with the given marker component.
///
/// Use as `cleanup_screen::<OnMyScreen>` when registering `OnExit` systems.
pub fn cleanup_screen<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
}

/// Spawns a 3D pushable button with edge + front layers.
///
/// The button wrapper is transparent and contains:
/// - An **edge** child (darker bg, stays in place) that peeks through at the bottom
/// - A **front** child (button face, offset upward) that slides on interaction
///
/// The front face moves up on hover and down on press, creating a physical depth illusion.
#[allow(clippy::too_many_arguments)]
pub fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    action: impl Component,
    style: &ButtonStyle,
) {
    let depth = -BUTTON_3D_OFFSET_REST; // positive value = edge visible at bottom

    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(style.width),
                height: Val::Px(style.height + depth),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderRadius::all(Val::Px(8.0)),
            BoxShadow(vec![ShadowStyle {
                color: BUTTON_SHADOW_COLOR,
                x_offset: Val::Px(0.0),
                y_offset: Val::Px(2.0),
                spread_radius: Val::Px(0.0),
                blur_radius: Val::Px(4.0),
            }]),
            ButtonColors {
                background: style.background,
                border: style.border,
            },
            ButtonAnimState {
                current: BUTTON_3D_OFFSET_REST,
                target: BUTTON_3D_OFFSET_REST,
            },
            action,
        ))
        .with_children(|wrapper| {
            // Edge layer — same size as front but offset down and slightly narrower.
            // Outline lives here so it appears to go behind the front face.
            wrapper.spawn((
                ButtonEdge,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(edge_color(style.background)),
                BorderRadius::all(Val::Px(8.0)),
                Outline::new(Val::Px(1.0), Val::Px(1.0), FRAME_OUTLINE_COLOR),
            ));

            // Front face — the interactive surface, offset upward.
            wrapper
                .spawn((
                    ButtonFront,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(style.height),
                        border: UiRect::all(Val::Px(style.border_width)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        overflow: Overflow::clip(),
                        position_type: PositionType::Relative,
                        top: Val::Px(BUTTON_3D_OFFSET_REST),
                        ..default()
                    },
                    BackgroundColor(opaque(style.background)),
                    BorderColor::all(style.border),
                    BorderRadius::all(Val::Px(8.0)),
                ))
                .with_children(|front| {
                    if style.text_shadow {
                        spawn_shadowed_text(
                            front,
                            text,
                            style.font_size,
                            style.text_color,
                            Node::default(),
                        );
                    } else {
                        front.spawn((
                            Text::new(text),
                            TextFont::from_font_size(style.font_size),
                            TextColor(style.text_color),
                            TextLayout::new_with_justify(Justify::Center),
                        ));
                    }
                });
        });
}

/// Spawns text with a drop shadow inside the given parent.
/// Uses a relative wrapper with an absolute-positioned shadow behind the main text.
/// Offset scales with font size (font_size / 20).
fn spawn_shadowed_text(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    font_size: f32,
    text_color: Color,
    node: Node,
) {
    let offset = font_size / 20.0;
    parent
        .spawn(Node {
            position_type: PositionType::Relative,
            ..node
        })
        .with_children(|wrapper| {
            wrapper.spawn((
                Text::new(text),
                TextFont::from_font_size(font_size),
                TextColor(TEXT_SHADOW_COLOR),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(offset),
                    top: Val::Px(offset),
                    width: Val::Percent(100.0),
                    ..default()
                },
            ));
            wrapper.spawn((
                Text::new(text),
                TextFont::from_font_size(font_size),
                TextColor(text_color),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

/// Spawns a standard page header row: title on the left, spacer, back button on the right.
pub fn spawn_page_header<B: Component>(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    font_size: f32,
    title_color: Color,
    back_action: B,
    button_style: &ButtonStyle,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        })
        .with_children(|header| {
            spawn_title_with_shadow(header, title, font_size, title_color, Node::default());
            header.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            spawn_button(header, "Back", back_action, button_style);
        });
}

/// Spawns a title text with a drop shadow effect.
pub fn spawn_title_with_shadow(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    font_size: f32,
    text_color: Color,
    node: Node,
) {
    spawn_shadowed_text(parent, text, font_size, text_color, node);
}

// ── UI Materials ──────────────────────────────────────────────────────────

/// Procedural parchment/stone texture material for panel backgrounds.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(crate) struct ParchmentMaterial {
    #[uniform(0)]
    pub data: ParchmentData,
}

#[derive(ShaderType, Debug, Clone, Copy)]
pub(crate) struct ParchmentData {
    pub base_color: LinearRgba,
    pub texture_strength: f32,
    pub vignette_strength: f32,
    pub noise_scale: f32,
    pub _padding: f32,
}

impl UiMaterial for ParchmentMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/parchment.wgsl".into()
    }
}

impl ParchmentMaterial {
    pub fn new(base_color: Color) -> Self {
        Self {
            data: ParchmentData {
                base_color: base_color.to_linear(),
                texture_strength: 0.45,
                vignette_strength: 0.4,
                noise_scale: 5.0,
                _padding: 0.0,
            },
        }
    }
}

/// Frosted glass overlay material for menu backgrounds.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(crate) struct FrostedGlassMaterial {
    #[uniform(0)]
    pub data: FrostedGlassData,
}

#[derive(ShaderType, Debug, Clone, Copy)]
pub(crate) struct FrostedGlassData {
    pub tint_color: LinearRgba,
    pub frost_intensity: f32,
    pub noise_scale: f32,
    pub _padding1: f32,
    pub _padding2: f32,
}

impl UiMaterial for FrostedGlassMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/frosted_glass.wgsl".into()
    }
}

impl FrostedGlassMaterial {
    pub fn new() -> Self {
        Self {
            data: FrostedGlassData {
                tint_color: Color::hsla(20.0, 0.04, 0.12, 0.30).to_linear(),
                frost_intensity: 1.0,
                noise_scale: 6.0,
                _padding1: 0.0,
                _padding2: 0.0,
            },
        }
    }
}

// ── Shared Slider ─────────────────────────────────────────────────────────

/// Configuration for spawning a slider row.
pub(crate) struct SliderRowConfig<
    'a,
    TText,
    TDownButton,
    TUpButton,
    TSliderTrack,
    TSliderFill,
    TSliderHandle,
> {
    pub label: &'a str,
    pub current_value: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub label_width: f32,
    pub text_component: TText,
    pub down_button: TDownButton,
    pub up_button: TUpButton,
    pub slider_track: TSliderTrack,
    pub slider_fill: TSliderFill,
    pub slider_handle: TSliderHandle,
}

/// Spawns a slider row with label, decrease/increase buttons, track, fill, handle, and value text.
/// Shared by settings and roguelite modifier screens.
pub(crate) fn spawn_slider_row<
    TText: Component,
    TDownButton: Component,
    TUpButton: Component,
    TSliderTrack: Component,
    TSliderFill: Component,
    TSliderHandle: Component,
>(
    parent: &mut ChildSpawnerCommands,
    config: SliderRowConfig<
        '_,
        TText,
        TDownButton,
        TUpButton,
        TSliderTrack,
        TSliderFill,
        TSliderHandle,
    >,
) {
    let SliderRowConfig {
        label,
        current_value,
        min_value,
        max_value,
        label_width,
        text_component,
        down_button,
        up_button,
        slider_track,
        slider_fill,
        slider_handle,
    } = config;

    let range = max_value - min_value;
    let normalized = if range > 0.0 {
        (current_value - min_value) / range
    } else {
        0.0
    };

    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(SLIDER_GAP),
            ..default()
        })
        .with_children(|row| {
            // Label (min_width ensures consistent alignment regardless of text length)
            row.spawn((
                Text::new(label),
                TextFont::from_font_size(SLIDER_LABEL_FONT_SIZE),
                TextColor(TEXT_PRIMARY),
                Node {
                    min_width: Val::Px(label_width),
                    width: Val::Px(label_width),
                    ..default()
                },
            ));

            // Controls
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(SLIDER_GAP),
                ..default()
            })
            .with_children(|controls| {
                // Decrease button
                controls
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(SLIDER_BUTTON_SIZE),
                            height: Val::Px(SLIDER_BUTTON_SIZE),
                            border: UiRect::all(Val::Px(SLIDER_BORDER_WIDTH)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor::all(SLIDER_BUTTON_BORDER_COLOR),
                        BorderRadius::all(Val::Px(4.0)),
                        BackgroundColor(SLIDER_BUTTON_BG),
                        ButtonColors {
                            background: SLIDER_BUTTON_BG,
                            border: SLIDER_BUTTON_BORDER_COLOR,
                        },
                        down_button,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("-"),
                            TextFont::from_font_size(SLIDER_BUTTON_FONT_SIZE),
                            TextColor(TEXT_PRIMARY),
                        ));
                    });

                // Slider track
                controls
                    .spawn((
                        Node {
                            width: Val::Px(SLIDER_TRACK_WIDTH),
                            height: Val::Px(12.0),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            position_type: PositionType::Relative,
                            ..default()
                        },
                        BorderColor::all(SLIDER_BUTTON_BORDER_COLOR),
                        BorderRadius::all(Val::Px(6.0)),
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                        Interaction::default(),
                        RelativeCursorPosition::default(),
                        slider_track,
                    ))
                    .with_children(|track| {
                        // Slider fill
                        track.spawn((
                            Node {
                                width: Val::Percent(normalized * 100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BorderRadius {
                                top_left: Val::Px(6.0),
                                bottom_left: Val::Px(6.0),
                                top_right: Val::Px(0.0),
                                bottom_right: Val::Px(0.0),
                            },
                            BackgroundColor(SLIDER_BUTTON_BORDER_COLOR),
                            slider_fill,
                        ));

                        // Slider handle (offset by -2px to center the 4px wide bar)
                        track.spawn((
                            Node {
                                width: Val::Px(4.0),
                                height: Val::Px(20.0),
                                position_type: PositionType::Absolute,
                                left: Val::Px(normalized * SLIDER_TRACK_WIDTH - 2.0),
                                top: Val::Px(-4.0),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(2.0)),
                            BackgroundColor(Color::WHITE),
                            BorderColor::all(SLIDER_BUTTON_BORDER_COLOR),
                            Interaction::default(),
                            RelativeCursorPosition::default(),
                            slider_handle,
                        ));
                    });

                // Increase button
                controls
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(SLIDER_BUTTON_SIZE),
                            height: Val::Px(SLIDER_BUTTON_SIZE),
                            border: UiRect::all(Val::Px(SLIDER_BORDER_WIDTH)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor::all(SLIDER_BUTTON_BORDER_COLOR),
                        BorderRadius::all(Val::Px(4.0)),
                        BackgroundColor(SLIDER_BUTTON_BG),
                        ButtonColors {
                            background: SLIDER_BUTTON_BG,
                            border: SLIDER_BUTTON_BORDER_COLOR,
                        },
                        up_button,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("+"),
                            TextFont::from_font_size(SLIDER_BUTTON_FONT_SIZE),
                            TextColor(TEXT_PRIMARY),
                        ));
                    });

                // Value display
                controls.spawn((
                    Text::new(format!("{}%", (current_value * 100.0) as u32)),
                    TextFont::from_font_size(SLIDER_LABEL_FONT_SIZE),
                    TextColor(TEXT_PRIMARY),
                    Node {
                        width: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    text_component,
                ));
            });
        });
}
