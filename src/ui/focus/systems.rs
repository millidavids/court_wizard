//! Focus navigation systems.

use bevy::prelude::*;
use bevy::ui::ComputedNode;
use bevy::ui::OverflowAxis;
use bevy::ui::ScrollPosition;
use bevy::ui::ui_transform::UiGlobalTransform;

use super::components::{
    CrossRowHorizontalNav, Focusable, GamepadFocused, GamepadScrollTarget, ModalOverlay,
    NoGamepadFocus, ScrollRevealBounds, TabFocusable,
};
use super::constants::{
    AUTOSCROLL_EDGE_PADDING, FOCUS_REPEAT_INITIAL_DELAY, FOCUS_REPEAT_INTERVAL,
    PANEL_COLUMN_TOLERANCE, RIGHT_STICK_SCROLL_SPEED, SAME_ROW_TOLERANCE,
    STICK_DIRECTION_THRESHOLD, STICK_RESET_THRESHOLD,
};
use super::resources::{FocusedEntity, PreModalFocus, ScreenFocusMemory, ScreenKey};
use crate::game::input::gamepad::messages::MenuBackPressed;
use crate::game::input::gamepad::resources::{ActiveInputDevice, GamepadAimSettings};
use crate::game::input::gamepad::systems::apply_deadzone_and_curve;
use crate::game::input::messages::MouseClicked;
use crate::state::{AppState, InGameState, MenuState, MetaGameState, PauseMenuState};
use crate::ui::components::ButtonActive;

fn screen_key(
    app: AppState,
    menu: Option<MenuState>,
    pause: Option<PauseMenuState>,
    meta: Option<MetaGameState>,
    in_game: Option<InGameState>,
) -> ScreenKey {
    ScreenKey {
        app,
        in_game,
        pause,
        menu,
        meta,
    }
}

type FocusableQuery<'w, 's, F> = Query<
    'w,
    's,
    (
        Entity,
        &'static UiGlobalTransform,
        Option<&'static InheritedVisibility>,
    ),
    F,
>;

/// Body-panel navigation excludes tab buttons (cycled via LB/RB) and header
/// back buttons (reachable only via the B/East button on gamepad).
type BodyFocusableQuery<'w, 's> = FocusableQuery<
    'w,
    's,
    (
        With<Focusable>,
        Without<TabFocusable>,
        Without<NoGamepadFocus>,
    ),
>;

/// Finds the nearest `Focusable` in the given direction.
///
/// Horizontal (Left/Right): prefers candidates in the same row (perp within
/// `SAME_ROW_TOLERANCE`), but falls back to cross-row candidates when no
/// same-row candidate exists — so pressing Right from a panel can jump to
/// the adjacent panel, picking the nearest-row item in the target.
///
/// Vertical (Up/Down): prefers candidates in the same panel column (X within
/// `PANEL_COLUMN_TOLERANCE`), then closest row bucket, then closest column
/// within that bucket. Prevents cross-panel jumps on Down when the current
/// panel still has items below.
fn nearest_in_direction(
    from: Vec2,
    direction: Vec2,
    relax_horizontal: bool,
    candidates: impl IntoIterator<Item = (Entity, Vec2)>,
) -> Option<Entity> {
    let horizontal = direction.x.abs() > direction.y.abs();
    let candidates: Vec<(Entity, Vec2)> = candidates.into_iter().collect();

    // Two-pass for vertical nav: first try same-panel-column candidates so
    // side-by-side panels (spell book / cauldron / compendium) don't bleed
    // into each other on Up/Down. If none exist, fall back to any column —
    // this is what lets row-based settings layouts (where row 1's controls
    // don't share X with row 2's controls) navigate Down to the next row
    // regardless of column alignment.
    if !horizontal {
        if let Some(found) = nearest_pass(from, direction, &candidates, true) {
            return Some(found);
        }
        return nearest_pass(from, direction, &candidates, false);
    }

    // Horizontal nav: single pass, gated by SAME_ROW_TOLERANCE unless the
    // focused entity opts into cross-row hops via CrossRowHorizontalNav.
    let mut best: Option<(Entity, f32, f32)> = None;
    for &(entity, pos) in &candidates {
        let delta = pos - from;
        let along = delta.dot(direction);
        if along <= 0.0 {
            continue;
        }
        let perp = (delta - direction * along).length();
        if !relax_horizontal && perp >= SAME_ROW_TOLERANCE {
            continue;
        }
        let is_better = match best {
            None => true,
            Some((_, best_along, best_perp)) => {
                if (along - best_along).abs() < SAME_ROW_TOLERANCE {
                    perp < best_perp
                } else {
                    along < best_along
                }
            }
        };
        if is_better {
            best = Some((entity, along, perp));
        }
    }
    best.map(|(e, _, _)| e)
}

fn nearest_pass(
    from: Vec2,
    direction: Vec2,
    candidates: &[(Entity, Vec2)],
    require_same_column: bool,
) -> Option<Entity> {
    let mut best: Option<(Entity, f32, f32)> = None;
    for &(entity, pos) in candidates {
        let delta = pos - from;
        let along = delta.dot(direction);
        if along <= 0.0 {
            continue;
        }
        let perp = (delta - direction * along).length();
        if require_same_column && delta.x.abs() >= PANEL_COLUMN_TOLERANCE {
            continue;
        }
        let is_better = match best {
            None => true,
            Some((_, best_along, best_perp)) => {
                if (along - best_along).abs() < SAME_ROW_TOLERANCE {
                    perp < best_perp
                } else {
                    along < best_along
                }
            }
        };
        if is_better {
            best = Some((entity, along, perp));
        }
    }
    best.map(|(e, _, _)| e)
}

/// Collects visible body focusables (tabs excluded) with their screen-space
/// centers. When any `ModalOverlay` is alive, restricts the set to focusables
/// that are descendants of a modal — blocking focus from escaping back to the
/// main screen underneath.
fn gather_focusables(
    query: &BodyFocusableQuery,
    modals: &Query<Entity, With<ModalOverlay>>,
    child_of: &Query<&ChildOf>,
) -> Vec<(Entity, Vec2)> {
    let modal_set: Vec<Entity> = modals.iter().collect();
    query
        .iter()
        .filter(|(_, _, vis)| vis.map(|v| v.get()).unwrap_or(true))
        .filter(|(e, _, _)| modal_set.is_empty() || is_descendant_of_any(*e, &modal_set, child_of))
        .map(|(e, transform, _)| (e, transform.translation))
        .collect()
}

fn is_descendant_of_any(entity: Entity, ancestors: &[Entity], child_of: &Query<&ChildOf>) -> bool {
    let mut current = entity;
    if ancestors.contains(&current) {
        return true;
    }
    while let Ok(c) = child_of.get(current) {
        let parent = c.parent();
        if ancestors.contains(&parent) {
            return true;
        }
        current = parent;
    }
    false
}

/// If the current focus is invalid (despawned, hidden, or on a tab button) or
/// unset while a gamepad is active, pick the top-leftmost body focusable.
pub(super) fn auto_refocus(
    active: Res<ActiveInputDevice>,
    focusables: BodyFocusableQuery,
    modals: Query<Entity, With<ModalOverlay>>,
    child_of: Query<&ChildOf>,
    mut focused: ResMut<FocusedEntity>,
) {
    if !active.is_gamepad() {
        return;
    }
    // When a modal is open, the current focus is valid only if it's inside
    // the modal. Otherwise fall through to re-pick inside the modal.
    if let Some(e) = focused.0
        && focusables
            .get(e)
            .is_ok_and(|(_, _, v)| v.map(|v| v.get()).unwrap_or(true))
    {
        let modal_set: Vec<Entity> = modals.iter().collect();
        if modal_set.is_empty() || is_descendant_of_any(e, &modal_set, &child_of) {
            return;
        }
    }
    let candidates = gather_focusables(&focusables, &modals, &child_of);
    focused.0 = candidates
        .into_iter()
        .min_by(|(_, a), (_, b)| {
            let key_a = (a.y, a.x);
            let key_b = (b.y, b.x);
            key_a
                .partial_cmp(&key_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(e, _)| e);
}

/// Tracks modal open/close transitions to save and restore focus.
///
/// When a `ModalOverlay` appears, stashes the current `FocusedEntity` into
/// `PreModalFocus` so `auto_refocus` can then yank focus into the modal.
/// When the last modal closes, restores the stashed entity (if still valid)
/// — this must run BEFORE `auto_refocus` so the restored entity survives
/// the validity check there.
pub(super) fn track_modal_focus_restore(
    modals: Query<Entity, With<ModalOverlay>>,
    focusables: BodyFocusableQuery,
    mut focused: ResMut<FocusedEntity>,
    mut stash: ResMut<PreModalFocus>,
    mut was_modal_active: Local<bool>,
) {
    let modal_active = modals.iter().next().is_some();
    if modal_active && !*was_modal_active {
        stash.0 = focused.0;
    } else if !modal_active && *was_modal_active {
        if let Some(prev) = stash.0
            && focusables
                .get(prev)
                .is_ok_and(|(_, _, v)| v.map(|v| v.get()).unwrap_or(true))
        {
            focused.0 = Some(prev);
        }
        stash.0 = None;
    }
    *was_modal_active = modal_active;
}

#[derive(Clone, Copy)]
pub(super) struct FocusHoldState {
    direction: Vec2,
    pressed_at: std::time::Duration,
    last_fired_at: std::time::Duration,
}

/// Stick latches direction once past `STICK_DIRECTION_THRESHOLD` and holds it
/// until the stick returns below `STICK_RESET_THRESHOLD` — prevents diagonal
/// wobble from flipping up↔right mid-hold.
fn resolve_nav_direction(gamepad: &Gamepad, stick_latched: &mut Option<Vec2>) -> Option<Vec2> {
    if gamepad.pressed(GamepadButton::DPadUp) {
        return Some(Vec2::new(0.0, -1.0));
    }
    if gamepad.pressed(GamepadButton::DPadDown) {
        return Some(Vec2::new(0.0, 1.0));
    }
    if gamepad.pressed(GamepadButton::DPadLeft) {
        return Some(Vec2::new(-1.0, 0.0));
    }
    if gamepad.pressed(GamepadButton::DPadRight) {
        return Some(Vec2::new(1.0, 0.0));
    }

    let lx = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
    let ly = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
    let mag = (lx * lx + ly * ly).sqrt();

    if mag < STICK_RESET_THRESHOLD {
        *stick_latched = None;
    } else if mag >= STICK_DIRECTION_THRESHOLD && stick_latched.is_none() {
        // stick-up = -Y in screen space
        *stick_latched = Some(if lx.abs() > ly.abs() {
            Vec2::new(lx.signum(), 0.0)
        } else {
            Vec2::new(0.0, -ly.signum())
        });
    }
    *stick_latched
}

/// Moves focus in response to D-pad + left-stick input. Tabs are excluded;
/// LB/RB cycle tabs via `tab_cycle`. A fresh press fires once immediately;
/// holding past `FOCUS_REPEAT_INITIAL_DELAY` then auto-repeats at
/// `FOCUS_REPEAT_INTERVAL`. Switching directions restarts the delay.
#[allow(clippy::too_many_arguments)]
pub(super) fn focus_navigation(
    time: Res<Time>,
    active: Res<ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    focusables: BodyFocusableQuery,
    modals: Query<Entity, With<ModalOverlay>>,
    cross_row: Query<Entity, With<CrossRowHorizontalNav>>,
    child_of: Query<&ChildOf>,
    mut focused: ResMut<FocusedEntity>,
    mut stick_latched: Local<Option<Vec2>>,
    mut hold_state: Local<Option<FocusHoldState>>,
) {
    let Some(gamepad_entity) = active.gamepad_entity() else {
        *stick_latched = None;
        *hold_state = None;
        return;
    };
    let Ok(gamepad) = gamepads.get(gamepad_entity) else {
        return;
    };

    let raw_direction = resolve_nav_direction(gamepad, &mut stick_latched);
    let now = time.elapsed();

    let fire_direction: Option<Vec2> = match raw_direction {
        None => {
            *hold_state = None;
            None
        }
        Some(dir) => match hold_state.as_mut() {
            None => {
                *hold_state = Some(FocusHoldState {
                    direction: dir,
                    pressed_at: now,
                    last_fired_at: now,
                });
                Some(dir)
            }
            Some(state) if state.direction != dir => {
                *state = FocusHoldState {
                    direction: dir,
                    pressed_at: now,
                    last_fired_at: now,
                };
                Some(dir)
            }
            Some(state) => {
                let held = now.saturating_sub(state.pressed_at);
                let since_last = now.saturating_sub(state.last_fired_at);
                if held >= FOCUS_REPEAT_INITIAL_DELAY && since_last >= FOCUS_REPEAT_INTERVAL {
                    state.last_fired_at = now;
                    Some(state.direction)
                } else {
                    None
                }
            }
        },
    };

    let Some(direction) = fire_direction else {
        return;
    };

    let candidates = gather_focusables(&focusables, &modals, &child_of);
    if candidates.is_empty() {
        return;
    }

    let current_pos = focused
        .0
        .and_then(|e| candidates.iter().find(|(c, _)| *c == e).map(|(_, p)| *p))
        .unwrap_or_else(|| candidates[0].1);

    let relax_horizontal = focused.0.is_some_and(|e| cross_row.contains(e));
    if let Some(next) = nearest_in_direction(
        current_pos,
        direction,
        relax_horizontal,
        candidates
            .iter()
            .copied()
            .filter(|(e, _)| Some(*e) != focused.0),
    ) && focused.0 != Some(next)
    {
        focused.0 = Some(next);
    }
}

/// LB/RB cycles the active tab by emitting a `MouseClicked` on the adjacent
/// tab button. Does NOT move `FocusedEntity` — focus stays on body panels
/// where the D-pad / stick live.
///
/// The "current" tab is the one with `ButtonActive`; if none is active,
/// cycle starts from the first tab in screen order.
pub(super) fn tab_cycle(
    active: Res<ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    tabs: Query<
        (
            Entity,
            &UiGlobalTransform,
            Option<&InheritedVisibility>,
            Has<ButtonActive>,
        ),
        With<TabFocusable>,
    >,
    mut clicks: MessageWriter<MouseClicked>,
) {
    let Some(gamepad_entity) = active.gamepad_entity() else {
        return;
    };
    let Ok(gamepad) = gamepads.get(gamepad_entity) else {
        return;
    };

    let forward = gamepad.just_pressed(GamepadButton::RightTrigger);
    let back = gamepad.just_pressed(GamepadButton::LeftTrigger);
    if !forward && !back {
        return;
    }

    let mut sorted: Vec<(Entity, Vec2, bool)> = tabs
        .iter()
        .filter(|(_, _, vis, _)| vis.map(|v| v.get()).unwrap_or(true))
        .map(|(e, transform, _, is_active)| (e, transform.translation, is_active))
        .collect();
    sorted
        .sort_by(|(_, a, _), (_, b, _)| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    if sorted.is_empty() {
        return;
    }

    let current_idx = sorted
        .iter()
        .position(|(_, _, active)| *active)
        .unwrap_or(0);

    let next_idx = if forward {
        (current_idx + 1) % sorted.len()
    } else {
        (current_idx + sorted.len() - 1) % sorted.len()
    };

    let next_entity = sorted[next_idx].0;
    clicks.write(MouseClicked {
        button: next_entity,
    });
}

/// Forces the focused entity's `Interaction` to `Hovered`/`Pressed` so the
/// existing button-animation and `MouseClicked` pipelines work on gamepad,
/// and maintains the `GamepadFocused` marker so focus visuals (purple tint)
/// can distinguish controller focus from mouse hover.
pub(super) fn override_focused_interaction(
    active: Res<ActiveInputDevice>,
    focused: Res<FocusedEntity>,
    gamepads: Query<&Gamepad>,
    mut commands: Commands,
    mut last_focused: Local<Option<Entity>>,
    mut confirm_armed: Local<bool>,
    mut interactions: Query<(Entity, &mut Interaction), (With<Focusable>, Without<NoGamepadFocus>)>,
) {
    let confirm_held = active
        .gamepad_entity()
        .and_then(|e| gamepads.get(e).ok())
        .is_some_and(|g| g.pressed(GamepadButton::South));

    let current = if active.is_gamepad() { focused.0 } else { None };

    if current != *last_focused {
        if let Some(prev) = *last_focused {
            commands.entity(prev).try_remove::<GamepadFocused>();
        }
        if let Some(new) = current {
            // try_insert: entity may have been despawned between command queue
            // and apply (state transitions, screen cleanup).
            commands.entity(new).try_insert(GamepadFocused);
        }
        *last_focused = current;
        // If A was held through this focus change (e.g. the press that
        // triggered a cursor→focus transition), disarm so the same press
        // doesn't immediately click the newly-focused button.
        if confirm_held {
            *confirm_armed = false;
        }
    }

    // Arm A once it's released, so the next press counts as a click.
    if !confirm_held {
        *confirm_armed = true;
    }

    if !active.is_gamepad() {
        return;
    }

    let active_confirm = confirm_held && *confirm_armed;

    // Clear ghost `Hovered` stamped by Bevy's `ui_focus_system` from the
    // hidden OS cursor position. Leave `Pressed` alone — clearing it would
    // fire a spurious `MouseClicked` via `button_click_detection`.
    for (entity, mut interaction) in &mut interactions {
        if Some(entity) == current {
            let desired = if active_confirm {
                Interaction::Pressed
            } else {
                Interaction::Hovered
            };
            interaction.set_if_neq(desired);
        } else if *interaction == Interaction::Hovered {
            interaction.set_if_neq(Interaction::None);
        }
    }
}

/// Clears stale focus when the back button is pressed (B / East / Start).
/// Avoids a leftover focus on a modal that just closed.
pub(super) fn clear_focus_on_back(
    mut back: MessageReader<MenuBackPressed>,
    mut focused: ResMut<FocusedEntity>,
) {
    if back.read().next().is_some() {
        focused.0 = None;
    }
}

/// Resets focus when the active input device switches away from gamepad.
pub(super) fn clear_focus_on_device_switch(
    active: Res<ActiveInputDevice>,
    mut focused: ResMut<FocusedEntity>,
) {
    if !active.is_gamepad() {
        focused.0 = None;
    }
}

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
/// Two Bevy 0.17 gotchas drive the implementation:
/// - `UiGlobalTransform.translation` IS scroll-adjusted (layout subtracts
///   the parent's scroll offset, see `bevy_ui/src/layout/mod.rs:236`), so
///   the focus's rendered position already reflects current scroll — we
///   compute an incremental delta against the container edges.
/// - `Node` requires `ScrollPosition` as a default component, so every UI
///   entity has one — to find the real scroll container we check for
///   `Node::overflow.y == OverflowAxis::Scroll`, not presence of
///   `ScrollPosition`.
///
/// Units: `ComputedNode` sizes and `UiGlobalTransform` are physical pixels;
/// `ScrollPosition` is logical. Convert via `inverse_scale_factor`.
#[allow(clippy::too_many_arguments)]
pub(super) fn autoscroll_to_focused(
    mut commands: Commands,
    focused: Res<FocusedEntity>,
    mut last_focus: Local<Option<Entity>>,
    nodes: Query<(&ComputedNode, &UiGlobalTransform)>,
    reveal_bounds: Query<Entity, With<ScrollRevealBounds>>,
    child_of_query: Query<&ChildOf>,
    scroll_containers: Query<(&ScrollPosition, &ComputedNode, &UiGlobalTransform, &Node)>,
    anim: Query<&ScrollAnimation>,
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
        if let Ok((scroll, container_node, container_xform, node)) = scroll_containers.get(parent)
            && node.overflow.y == OverflowAxis::Scroll
        {
            let inv_sf = container_node.inverse_scale_factor();
            if inv_sf <= 0.0 {
                break;
            }

            let Ok((reveal_node, reveal_xform)) = nodes.get(reveal_entity) else {
                break;
            };
            let reveal_size_y = reveal_node.size().y;
            let reveal_center_y = reveal_xform.translation.y;
            let reveal_top = reveal_center_y - reveal_size_y * 0.5;
            let reveal_bottom = reveal_center_y + reveal_size_y * 0.5;

            let container_size_y = container_node.size().y;
            let content_size_y = container_node.content_size().y;
            let container_center_y = container_xform.translation.y;
            let container_top = container_center_y - container_size_y * 0.5;
            let container_bottom = container_center_y + container_size_y * 0.5;

            // Use any in-flight animation target as the "current" scroll so
            // successive focus changes mid-animation don't fight each other.
            let current_scroll = anim.get(parent).map(|a| a.target_y).unwrap_or(scroll.y);
            let max_scroll_logical = (content_size_y - container_size_y).max(0.0) * inv_sf;
            let padding_physical = AUTOSCROLL_EDGE_PADDING;

            let new_target = if reveal_top < container_top + padding_physical {
                let delta_physical = (container_top + padding_physical) - reveal_top;
                (current_scroll - delta_physical * inv_sf).max(0.0)
            } else if reveal_bottom > container_bottom - padding_physical {
                let delta_physical = reveal_bottom - (container_bottom - padding_physical);
                (current_scroll + delta_physical * inv_sf).min(max_scroll_logical)
            } else {
                break; // already visible
            };

            commands.entity(parent).insert(ScrollAnimation {
                target_y: new_target,
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
    aim: Res<GamepadAimSettings>,
    gamepads: Query<&Gamepad>,
    mut commands: Commands,
    mut targets: Query<(Entity, &mut ScrollPosition, &ComputedNode), With<GamepadScrollTarget>>,
) {
    let Some(gamepad_entity) = active.gamepad_entity() else {
        return;
    };
    let Ok(gamepad) = gamepads.get(gamepad_entity) else {
        return;
    };
    let ry = gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0);
    // Stick up (positive Y) scrolls content up (decrease scroll.y); negate so
    // the mapping feels natural. Route through `apply_deadzone_and_curve` so
    // scroll uses the same deadzone + response curve as the virtual cursor.
    let shaped = apply_deadzone_and_curve(Vec2::new(0.0, -ry), aim.deadzone, aim.response_curve).y;
    if shaped == 0.0 {
        return;
    }
    let delta = shaped * RIGHT_STICK_SCROLL_SPEED * time.delta_secs();
    for (entity, mut scroll, node) in &mut targets {
        let max = (node.content_size().y - node.size().y).max(0.0);
        scroll.y = (scroll.y + delta).clamp(0.0, max);
        // Manual scroll overrides any in-flight autoscroll animation.
        commands.entity(entity).try_remove::<ScrollAnimation>();
    }
}

/// Records the focused entity's screen position under the current screen's
/// key whenever focus changes. On re-entry to that screen, the position is
/// used by `restore_focus_on_screen_change` to snap focus to the nearest
/// focusable rather than the default top-leftmost.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_focus_memory(
    app: Res<State<AppState>>,
    menu: Option<Res<State<MenuState>>>,
    pause: Option<Res<State<PauseMenuState>>>,
    meta: Option<Res<State<MetaGameState>>>,
    in_game: Option<Res<State<InGameState>>>,
    focused: Res<FocusedEntity>,
    focusables: BodyFocusableQuery,
    mut memory: ResMut<ScreenFocusMemory>,
    mut last_focused: Local<Option<Entity>>,
) {
    if focused.0 == *last_focused {
        return;
    }
    *last_focused = focused.0;

    let Some(entity) = focused.0 else { return };
    let Ok((_, transform, vis)) = focusables.get(entity) else {
        return;
    };
    if !vis.map(|v| v.get()).unwrap_or(true) {
        return;
    }
    let key = screen_key(
        *app.get(),
        menu.as_deref().map(|s| *s.get()),
        pause.as_deref().map(|s| *s.get()),
        meta.as_deref().map(|s| *s.get()),
        in_game.as_deref().map(|s| *s.get()),
    );
    memory.0.insert(key, transform.translation);
}

/// On screen change, restore focus to the focusable nearest the last-saved
/// position for that screen. If no position is saved, leave focus unset so
/// `auto_refocus` picks the default top-leftmost.
#[allow(clippy::too_many_arguments)]
pub(super) fn restore_focus_on_screen_change(
    app: Res<State<AppState>>,
    menu: Option<Res<State<MenuState>>>,
    pause: Option<Res<State<PauseMenuState>>>,
    meta: Option<Res<State<MetaGameState>>>,
    in_game: Option<Res<State<InGameState>>>,
    memory: Res<ScreenFocusMemory>,
    focusables: BodyFocusableQuery,
    modals: Query<Entity, With<ModalOverlay>>,
    child_of: Query<&ChildOf>,
    mut focused: ResMut<FocusedEntity>,
    mut last_key: Local<Option<ScreenKey>>,
) {
    let key = screen_key(
        *app.get(),
        menu.as_deref().map(|s| *s.get()),
        pause.as_deref().map(|s| *s.get()),
        meta.as_deref().map(|s| *s.get()),
        in_game.as_deref().map(|s| *s.get()),
    );
    if *last_key == Some(key) {
        return;
    }

    let candidates = gather_focusables(&focusables, &modals, &child_of);
    if candidates.is_empty() {
        // Entities might not be spawned yet — retry next frame by not
        // updating last_key.
        return;
    }
    *last_key = Some(key);

    let Some(&saved_pos) = memory.0.get(&key) else {
        return;
    };
    if let Some(&(nearest, _)) = candidates.iter().min_by(|(_, a), (_, b)| {
        a.distance_squared(saved_pos)
            .partial_cmp(&b.distance_squared(saved_pos))
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        focused.0 = Some(nearest);
    }
}
