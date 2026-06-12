use bevy::prelude::*;
use bevy::ui::ShadowStyle;

use super::super::constants::{
    CONTENT_BG, CONTENT_BORDER, DETAIL_BG, DETAIL_BORDER, DETAIL_PADDING, FRAME_OUTER_RING_COLOR,
    FRAME_OUTLINE_COLOR, FRAME_OUTLINE_OFFSET, FRAME_OUTLINE_WIDTH, FRAME_SHADOW_SPREAD_BASE,
    LEFT_PANEL_WIDTH, LIST_BG, LIST_BORDER, OVERLAY_BG, PANEL_BORDER_RADIUS, PANEL_SHADOW_AMBIENT,
    PANEL_SHADOW_CONTACT, PANEL_SHADOW_MEDIUM, SCROLL_BG, SCROLL_BORDER, SCROLL_SHADOW_COLOR,
    SHADOW_COLOR,
};
use super::materials::FrostedGlassOverlay;

/// Returns the shared styling bundle for inner scrollable areas, with
/// scroll-area border-radius baked into the supplied `Node`. Callers add
/// `ScrollPosition::default()` and their marker component alongside.
pub(crate) fn scroll_area_style(mut node: Node) -> (Node, BackgroundColor, BorderColor, BoxShadow) {
    node.border_radius = BorderRadius::all(Val::Px(4.0));
    (
        node,
        BackgroundColor(SCROLL_BG),
        BorderColor::all(SCROLL_BORDER),
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
                        border_radius: BorderRadius::all(Val::Px(PANEL_BORDER_RADIUS)),
                        ..default()
                    },
                    BackgroundColor(DETAIL_BG),
                    BorderColor::all(DETAIL_BORDER),
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
    parent.commands().entity(detail_box).insert((
        marker,
        ScrollPosition::default(),
        crate::ui::focus::GamepadScrollTarget,
    ));
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
                // Required: flex's implicit `min-height: auto` would let this
                // item grow to its content size, defeating `scroll_y()`.
                min_height: Val::Px(0.0),
                flex_direction: inner_direction,
                row_gap: Val::Px(inner_gap),
                column_gap: Val::Px(inner_gap),
                overflow: Overflow::scroll_y(),
                // Default `Stretch` would force children to panel height,
                // again defeating scroll_y.
                align_items: AlignItems::FlexStart,
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(12.0)),
                border_radius: BorderRadius::all(Val::Px(PANEL_BORDER_RADIUS)),
                ..default()
            },
            BackgroundColor(LIST_BG),
            BorderColor::all(LIST_BORDER),
            ScrollPosition::default(),
            crate::ui::focus::GamepadScrollTarget,
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
/// When `pause_menu` is true, `GlobalZIndex(1000)` is added so the page
/// renders above all other UI (e.g., spell book / cauldron menus). When
/// `pause_menu` is false, `GlobalZIndex(500)` is used instead.
pub fn spawn_page_container<M: Component>(
    commands: &mut Commands,
    screen_marker: M,
    pause_menu: bool,
    mut content_node: Node,
) -> Entity {
    content_node.border_radius = BorderRadius::all(Val::Px(6.0));
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

    // Modal overlays sit above the in-game HUD (100..). Pause goes highest
    // since it can open on top of the spell book / cauldron menus.
    root.insert(GlobalZIndex(if pause_menu { 1000 } else { 500 }));

    let root_id = root.id();

    let content_id = commands
        .spawn((
            content_node,
            BackgroundColor(CONTENT_BG),
            BorderColor::all(CONTENT_BORDER),
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
                    color: PANEL_SHADOW_CONTACT,
                    x_offset: Val::Px(0.0),
                    y_offset: Val::Px(1.0),
                    spread_radius: Val::Px(FRAME_SHADOW_SPREAD_BASE),
                    blur_radius: Val::Px(3.0),
                },
                // Medium depth shadow
                ShadowStyle {
                    color: PANEL_SHADOW_MEDIUM,
                    x_offset: Val::Px(0.0),
                    y_offset: Val::Px(4.0),
                    spread_radius: Val::Px(FRAME_SHADOW_SPREAD_BASE + 2.0),
                    blur_radius: Val::Px(8.0),
                },
                // Wide ambient shadow
                ShadowStyle {
                    color: PANEL_SHADOW_AMBIENT,
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
