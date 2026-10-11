//! Controller confirm: A on the focused button presses and activates it — the
//! same `Pressed` + `Activate` path a mouse click takes.

use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::Activate;

use super::components::{Focusable, GamepadFocused, NoGamepadFocus};
use super::resources::FocusedEntity;
use crate::game::input::action_state::{GamepadAction, GamepadActionState};
use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::ui::components::UiButton;

/// Keeps `GamepadFocused` on the focused entity and turns A into button presses.
///
/// A fresh A press on the focused button presses it; releasing A activates it.
/// A focus or device change mid-press cancels without activating.
pub(super) fn drive_gamepad_focus(
    active: Res<ActiveInputDevice>,
    state: Res<GamepadActionState>,
    focused: Res<FocusedEntity>,
    buttons: Query<(), (With<UiButton>, With<Focusable>, Without<NoGamepadFocus>)>,
    mut commands: Commands,
    mut last_focused: Local<Option<Entity>>,
    mut held: Local<Option<Entity>>,
) {
    let gamepad = active.is_gamepad();
    let current = if gamepad { focused.0 } else { None };
    let focus_changed = current != *last_focused;

    if focus_changed {
        if let Some(prev) = *last_focused {
            commands.entity(prev).try_remove::<GamepadFocused>();
        }
        if let Some(new) = current {
            // try_insert: entity may have been despawned between command queue
            // and apply (state transitions, screen cleanup).
            commands.entity(new).try_insert(GamepadFocused);
        }
        *last_focused = current;
    }

    if let Some(entity) = *held
        && Some(entity) != current
    {
        commands.entity(entity).try_remove::<Pressed>();
        *held = None;
    }

    // The press that moved focus (e.g. a cursor→focus transition) must not
    // also click the newly focused button.
    if held.is_none() && !focus_changed && gamepad && state.just_pressed(GamepadAction::UIConfirm) {
        if let Some(entity) = current.filter(|&e| buttons.contains(e)) {
            commands.entity(entity).try_insert(Pressed);
            *held = Some(entity);
        }
    } else if let Some(entity) = *held
        && !state.pressed(GamepadAction::UIConfirm)
    {
        commands.entity(entity).try_remove::<Pressed>();
        commands.trigger(Activate { entity });
        *held = None;
    }
}
