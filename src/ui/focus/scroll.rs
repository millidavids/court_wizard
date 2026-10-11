//! Focus-driven scrolling: autoscroll to the focused item and right-stick scroll.

use bevy::prelude::*;
use bevy::ui::ComputedNode;
use bevy::ui::OverflowAxis;
use bevy::ui::ScrollPosition;
use bevy::ui::ui_transform::UiGlobalTransform;

use super::components::{GamepadScrollTarget, ScrollRevealBounds};
use super::constants::{ANIM_CANCEL_THRESHOLD, AUTOSCROLL_EDGE_PADDING, RIGHT_STICK_SCROLL_SPEED};
use super::resources::FocusedEntity;
use crate::game::input::action_state::GamepadActionState;
use crate::game::input::gamepad::resources::{ActiveInputDevice, GamepadAimSettings};
use crate::game::input::gamepad::systems::apply_deadzone_and_curve;

/// Target scroll position in LOGICAL pixels, set by `autoscroll_to_focused`
/// and consumed by `animate_scroll` to tween `ScrollPosition.y` toward it.
/// Removed when the animation converges or the user grabs the scroll with
/// the right stick.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct ScrollAnimation {
    target_y: f32,
}

/// Scrolls the focused entity's nearest scrollable ancestor so the item is
/// visible within the viewport. If any ancestor between the focus and the
/// scroll container carries `ScrollRevealBounds`, that ancestor's rect is
/// used instead — so focusing a button at the bottom of a tall card still
/// reveals the card's header.
///
/// Anchors all math to `ComputedNode.scroll_position` (the physical scroll
/// value layout actually used) rather than `ScrollPosition.y` — the latter
/// can drift past Bevy's clamp / floor or out of sync with the layout that
/// produced `UiGlobalTransform`, which would yield a mismatched delta and
/// undershoot the snap for far-off-screen items.
///
/// `Node` requires `ScrollPosition` as a default component so every UI
/// entity has one; we identify the real scroll container by checking
/// `Node::overflow.y == OverflowAxis::Scroll`. `ComputedNode` sizes and
/// `UiGlobalTransform` are physical pixels; `ScrollPosition` is logical
/// — convert via `inverse_scale_factor`.
#[allow(clippy::too_many_arguments)]
pub(super) fn autoscroll_to_focused(
    mut commands: Commands,
    focused: Res<FocusedEntity>,
    mut last_focus: Local<Option<Entity>>,
    nodes: Query<(&ComputedNode, &UiGlobalTransform)>,
    reveal_bounds: Query<Entity, With<ScrollRevealBounds>>,
    child_of_query: Query<&ChildOf>,
    scroll_containers: Query<(&ComputedNode, &UiGlobalTransform, &Node), With<ScrollPosition>>,
) {
    if focused.0 == *last_focus {
        return;
    }
    *last_focus = focused.0;

    let Some(entity) = focused.0 else { return };

    // Walk up from focus to find (a) the rect to reveal — nearest ancestor
    // with `ScrollRevealBounds`, falling back to the focus entity itself —
    // and (b) the scroll container (first ancestor with `Overflow::scroll_y()`).
    let mut reveal_entity = entity;
    let mut current = entity;
    while let Ok(child_of) = child_of_query.get(current) {
        let parent = child_of.parent();
        if let Ok((container_node, container_xform, node)) = scroll_containers.get(parent)
            && node.overflow.y == OverflowAxis::Scroll
        {
            let inv_sf = container_node.inverse_scale_factor();
            if inv_sf <= 0.0 {
                break;
            }

            let Ok((reveal_node, reveal_xform)) = nodes.get(reveal_entity) else {
                break;
            };

            // All in physical pixels:
            let reveal_size_y = reveal_node.size().y;
            let reveal_top_screen = reveal_xform.translation.y - reveal_size_y * 0.5;

            let container_size_y = container_node.size().y;
            let content_size_y = container_node.content_size().y;
            let container_top_screen = container_xform.translation.y - container_size_y * 0.5;

            // The scroll value layout actually applied this frame. The
            // focus's screen position above was computed against this, so
            // adding it back recovers the focus's unscrolled (content-space)
            // top relative to the container's content top.
            let resolved_scroll_phys = container_node.scroll_position.y;
            let focus_top_content = reveal_top_screen + resolved_scroll_phys - container_top_screen;
            let focus_bottom_content = focus_top_content + reveal_size_y;

            let viewport_top_content = resolved_scroll_phys;
            let viewport_bottom_content = resolved_scroll_phys + container_size_y;
            let max_scroll_phys = (content_size_y - container_size_y).max(0.0);
            let padding_physical = AUTOSCROLL_EDGE_PADDING;

            let new_scroll_phys = if reveal_entity == entity && reveal_size_y >= container_size_y {
                // The FOCUSED element itself is taller than the viewport (e.g.
                // the controller-binding diagram): pin its TOP so the header
                // stays visible instead of pinning the bottom (which would push
                // the header off-screen above). Scoped to `reveal_entity ==
                // entity`: when a `ScrollRevealBounds` ancestor (e.g. an
                // expanded wizard card) is the tall reveal target, fall through
                // to the bottom-pin branch so the focused button stays visible.
                (focus_top_content - padding_physical).clamp(0.0, max_scroll_phys)
            } else if focus_top_content < viewport_top_content + padding_physical {
                // Focus above viewport — pin its top to viewport top + pad.
                (focus_top_content - padding_physical).max(0.0)
            } else if focus_bottom_content > viewport_bottom_content - padding_physical {
                // Focus below viewport — pin its bottom to viewport bottom - pad.
                (focus_bottom_content - container_size_y + padding_physical).min(max_scroll_phys)
            } else {
                break; // already visible
            };

            commands.entity(parent).insert(ScrollAnimation {
                target_y: new_scroll_phys * inv_sf,
            });
            break;
        }
        if reveal_bounds.contains(parent) && reveal_entity == entity {
            reveal_entity = parent;
        }
        current = parent;
    }
}

/// Exponential lerp of `ScrollPosition.y` toward `ScrollAnimation.target_y`.
/// When the distance drops below 0.5 logical pixels, snap and remove the
/// animation component. Speed constant `SCROLL_ANIM_SPEED` is in 1/sec
/// (higher = snappier).
pub(super) fn animate_scroll(
    time: Res<Time>,
    mut commands: Commands,
    mut animating: Query<(Entity, &mut ScrollPosition, &ScrollAnimation)>,
) {
    const SCROLL_ANIM_SPEED: f32 = 18.0;
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    let factor = 1.0 - (-SCROLL_ANIM_SPEED * dt).exp();
    for (entity, mut scroll, anim) in &mut animating {
        let diff = anim.target_y - scroll.y;
        if diff.abs() < 0.5 {
            scroll.y = anim.target_y;
            commands.entity(entity).remove::<ScrollAnimation>();
        } else {
            scroll.y += diff * factor;
        }
    }
}

/// Drives `ScrollPosition` of every `GamepadScrollTarget` container with the
/// right-stick Y axis. For text-heavy screens (Manual, Credits) and detail
/// panels the user scrolls with right stick up/down while focus stays where
/// it is.
pub(super) fn right_stick_scroll(
    time: Res<Time>,
    active: Res<ActiveInputDevice>,
    state: Res<GamepadActionState>,
    aim: Res<GamepadAimSettings>,
    mut commands: Commands,
    mut targets: Query<(Entity, &mut ScrollPosition, &ComputedNode), With<GamepadScrollTarget>>,
) {
    if !active.is_gamepad() {
        return;
    }
    let ry = state.right_stick.y;
    // Stick up (positive Y) scrolls content up (decrease scroll.y); negate so
    // the mapping feels natural. Route through `apply_deadzone_and_curve` so
    // scroll uses the same deadzone + response curve as the virtual cursor.
    let shaped = apply_deadzone_and_curve(Vec2::new(0.0, -ry), aim.deadzone, aim.response_curve).y;
    if shaped == 0.0 {
        return;
    }
    let delta = shaped * RIGHT_STICK_SCROLL_SPEED * time.delta_secs();
    // Manual scroll overrides any in-flight autoscroll animation, but only
    // when the stick is meaningfully deflected. Just past the deadzone the
    // shaped value can be ~0.001 (response curve flattens the boundary), and
    // a thumb resting on the stick at that level produces near-zero scroll
    // movement but would still strip a freshly-inserted `ScrollAnimation`
    // every frame — leaving navigation autoscroll stuck on its first lerp
    // tick. Require a clear push before claiming control.
    let cancel_anim = shaped.abs() >= ANIM_CANCEL_THRESHOLD;
    for (entity, mut scroll, node) in &mut targets {
        let max = (node.content_size().y - node.size().y).max(0.0);
        scroll.y = (scroll.y + delta).clamp(0.0, max);
        if cancel_anim {
            commands.entity(entity).try_remove::<ScrollAnimation>();
        }
    }
}
