//! Focus navigation: helpers, direction resolution, focus selection.

use bevy::prelude::*;
use bevy::ui::ui_transform::UiGlobalTransform;

use super::components::{
    ConsumeHorizontalNav, CrossRowHorizontalNav, DisabledTab, Focusable, ModalOverlay,
    NoGamepadFocus, TabFocusable,
};
use super::constants::{
    FOCUS_REPEAT_INITIAL_DELAY, FOCUS_REPEAT_INTERVAL, PANEL_COLUMN_TOLERANCE, SAME_ROW_TOLERANCE,
    STICK_DIRECTION_THRESHOLD, STICK_RESET_THRESHOLD,
};
use super::resources::{FocusedEntity, PreModalFocus, ScreenKey};
use crate::game::input::action_state::{GamepadAction, GamepadActionState};
use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::game::input::messages::MouseClicked;
use crate::state::{AppState, InGameState, MenuState, MetaGameState, PauseMenuState};
use crate::ui::components::ButtonActive;

pub(super) fn screen_key(
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
pub(super) type BodyFocusableQuery<'w, 's> = FocusableQuery<
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
pub(super) fn gather_focusables(
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
fn resolve_nav_direction(
    state: &GamepadActionState,
    stick_latched: &mut Option<Vec2>,
) -> Option<Vec2> {
    if state.pressed(GamepadAction::AbilityUp) {
        return Some(Vec2::new(0.0, -1.0));
    }
    if state.pressed(GamepadAction::AbilityDown) {
        return Some(Vec2::new(0.0, 1.0));
    }
    if state.pressed(GamepadAction::AbilityLeft) {
        return Some(Vec2::new(-1.0, 0.0));
    }
    if state.pressed(GamepadAction::AbilityRight) {
        return Some(Vec2::new(1.0, 0.0));
    }

    let lx = state.left_stick.x;
    let ly = state.left_stick.y;
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
    state: Res<GamepadActionState>,
    focusables: BodyFocusableQuery,
    modals: Query<Entity, With<ModalOverlay>>,
    cross_row: Query<Entity, With<CrossRowHorizontalNav>>,
    consume_horizontal: Query<Entity, With<ConsumeHorizontalNav>>,
    child_of: Query<&ChildOf>,
    mut focused: ResMut<FocusedEntity>,
    mut stick_latched: Local<Option<Vec2>>,
    mut hold_state: Local<Option<FocusHoldState>>,
) {
    if !active.is_gamepad() {
        *stick_latched = None;
        *hold_state = None;
        return;
    }

    let raw_direction = resolve_nav_direction(&state, &mut stick_latched);

    // A focused `ConsumeHorizontalNav` element (the controller-binding diagram)
    // eats Left/Right so it can repurpose them — e.g. cycle vendor schemes —
    // without focus hopping to a neighbor. Up/Down still navigate away, so it's
    // never a trap. Checked before the hold-repeat update below so consumed
    // presses don't advance the repeat timer.
    if let Some(dir) = raw_direction
        && dir.x.abs() > dir.y.abs()
        && focused.0.is_some_and(|e| consume_horizontal.contains(e))
    {
        return;
    }

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
    state: Res<GamepadActionState>,
    tabs: Query<
        (
            Entity,
            &UiGlobalTransform,
            Option<&InheritedVisibility>,
            Has<ButtonActive>,
        ),
        // Skip disabled tabs (e.g. the VS tab while a match/connection blocks it)
        // so bumper cycling steps over them instead of stalling — matching the
        // click handler, which also ignores `DisabledTab`.
        (With<TabFocusable>, Without<DisabledTab>),
    >,
    mut clicks: MessageWriter<MouseClicked>,
) {
    if !active.is_gamepad() {
        return;
    }

    let forward = state.just_pressed(GamepadAction::TabNext);
    let back = state.just_pressed(GamepadAction::TabPrev);
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
