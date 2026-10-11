//! Focus clearing and per-screen focus memory.

use bevy::prelude::*;

use super::components::ModalOverlay;
use super::navigation::{BodyFocusableQuery, gather_focusables, screen_key};
use super::resources::{FocusedEntity, ScreenFocusMemory, ScreenKey};
use crate::game::input::gamepad::messages::MenuBackPressed;
use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::state::{AppState, InGameState, MenuState, MetaGameState, PauseMenuState};

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
