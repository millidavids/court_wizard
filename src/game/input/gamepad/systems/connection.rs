use std::collections::HashMap;

use bevy::input::gamepad::GamepadConnectionEvent;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::prelude::*;
use bevy::window::{CursorMoved, CursorOptions, PrimaryWindow};

use super::super::arbiter::{InputDeviceArbiter, StickIntentTracker};
use super::super::resources::{ActiveInputDevice, VirtualCursorPosition};
use crate::config::GameConfig;
use crate::game::input::action_state::GamepadActionState;
use crate::game::input::components::MouseButtonState;
use crate::game::multiplayer::pause_request::RequestGamePauseMessage;

/// The most recent gilrs pad that drove input. Unlike [`ActiveInputDevice`] this
/// survives switching back to mouse/keyboard, so unplugging the pad the player
/// was using moments ago still pauses. Never cleared — a stale entity id can't
/// match a later pad's connection events, so it just stops matching.
#[derive(Resource, Default)]
pub(crate) struct LastActiveGamepad(pub(crate) Option<Entity>);

/// Updates `ActiveInputDevice` each frame based on which input source was used.
///
/// Rules (shared with the Steam Input poll via [`InputDeviceArbiter`]):
/// - A pad button edge is unambiguous intent: it claims focus immediately
///   (except on a frame that itself had mouse/keyboard events).
/// - Stick deflection is weak intent: measured against a tracked resting
///   baseline (never absolute position — a drifting pad idling on the desk
///   used to pass the old absolute check on every mouse-quiet frame, flapping
///   the device state several times a second) and gated behind the grace
///   window of mouse/keyboard silence.
/// - Mouse/keyboard events reclaim focus, debounced against held pad inputs:
///   while the active pad holds a button or stick, takeover needs two
///   consecutive event frames, so one stray cursor event can't cancel a
///   hold-to-cast mid-cast (real mouse use emits events every frame and takes
///   over within ~2 frames). A cycling phantom button press can in principle
///   still claim focus during a mouse pause — accepted: deliberate presses
///   must never be dropped, and takeover back is immediate.
#[allow(clippy::too_many_arguments)]
pub(crate) fn detect_active_input_device(
    mut active: ResMut<ActiveInputDevice>,
    mut last_gamepad: ResMut<LastActiveGamepad>,
    mut cursor_moved: MessageReader<CursorMoved>,
    mut mouse_buttons: MessageReader<MouseButtonInput>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut keyboard: MessageReader<KeyboardInput>,
    mut connections: MessageReader<GamepadConnectionEvent>,
    gamepads: Query<(Entity, &Gamepad)>,
    steam_actions: Res<GamepadActionState>,
    time: Res<Time<Real>>,
    mut arbiter: ResMut<InputDeviceArbiter>,
    mut prev_mk_frame: Local<bool>,
    mut trackers: Local<HashMap<Entity, StickIntentTracker>>,
) {
    let now = time.elapsed_secs_f64();
    let dt = time.delta_secs();

    // Full drains (count, not next) — leftover events must not masquerade as
    // fresh activity next frame; this feeds a time-window decision.
    let mk_activity = cursor_moved.read().count()
        + mouse_buttons.read().count()
        + mouse_wheel.read().count()
        + keyboard.read().count()
        > 0;
    let was_mk_frame = *prev_mk_frame;
    *prev_mk_frame = mk_activity;
    arbiter.mk_event_this_frame = mk_activity;
    if mk_activity {
        arbiter.last_mk_activity = Some(now);
    }

    // Any (dis)connect invalidates that pad's tracked rest position — event
    // based, so a disconnect+reconnect drained in a single frame (no component
    // gap for retain() to see) still re-seeds the baseline.
    for event in connections.read() {
        trackers.remove(&event.gamepad);
    }
    trackers.retain(|e, _| gamepads.contains(*e));

    // Assess every pad every frame: baselines must keep absorbing drift even
    // while the mouse is active. Buttons are checked independently of the
    // tracker so a first-sight frame (baseline seeding) can't swallow the
    // press that woke a wireless pad.
    let mut button_intent: Option<Entity> = None;
    let mut stick_intent: Option<Entity> = None;
    for (entity, gamepad) in &gamepads {
        let axes = [
            GamepadAxis::LeftStickX,
            GamepadAxis::LeftStickY,
            GamepadAxis::RightStickX,
            GamepadAxis::RightStickY,
        ]
        .map(|ax| gamepad.get(ax).unwrap_or(0.0));

        if trackers.entry(entity).or_default().assess(axes, dt) && stick_intent.is_none() {
            stick_intent = Some(entity);
        }
        if gamepad.get_just_pressed().next().is_some() && button_intent.is_none() {
            button_intent = Some(entity);
        }
    }

    // Record observed intent for the disconnect auto-pause safety net even when
    // the claim below is suppressed — a pad tapped moments after mouse use must
    // still count as "the pad the player was using" if it then dies.
    if let Some(entity) = button_intent.or(stick_intent)
        && last_gamepad.0 != Some(entity)
    {
        last_gamepad.0 = Some(entity);
    }

    if mk_activity {
        // Debounce against held pad inputs: a single stray event while the
        // active pad is holding a button or a deflected stick is swallowed.
        let gilrs_holding = active.gamepad_entity().is_some_and(|e| {
            gamepads
                .get(e)
                .is_ok_and(|(_, gp)| gp.get_pressed().next().is_some())
                || trackers.get(&e).is_some_and(StickIntentTracker::deflected)
        });
        let steam_holding = matches!(*active, ActiveInputDevice::SteamInputPad)
            && (steam_actions.buttons.get_pressed().next().is_some()
                || arbiter.steam_stick.deflected());
        let holding = gilrs_holding || steam_holding;

        if (!holding || was_mk_frame) && !matches!(*active, ActiveInputDevice::MouseKeyboard) {
            *active = ActiveInputDevice::MouseKeyboard;
        }
        // Never claim for a pad on a mouse/keyboard event frame.
        return;
    }

    // Buttons claim immediately; stick intent additionally needs the grace
    // window of mouse/keyboard silence.
    let claimant = button_intent.or_else(|| stick_intent.filter(|_| arbiter.mk_quiet(now)));
    let Some(entity) = claimant else { return };
    let should_switch = match *active {
        ActiveInputDevice::Gamepad(e) => e != entity,
        ActiveInputDevice::MouseKeyboard | ActiveInputDevice::SteamInputPad => true,
    };
    if should_switch {
        *active = ActiveInputDevice::Gamepad(entity);
    }
}

/// Hides the OS cursor when gamepad becomes active, shows it when mouse takes over.
/// Also clears stale mouse state on transition so a lingering left-click doesn't
/// accidentally fire a spell after switching to controller.
pub(crate) fn toggle_cursor_visibility(
    active: Res<ActiveInputDevice>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut last: Local<ActiveInputDevice>,
    mut virtual_cursor: ResMut<VirtualCursorPosition>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut mouse_state: ResMut<MouseButtonState>,
) {
    if *last == *active {
        return;
    }
    *last = *active;

    let Ok(mut cursor) = cursor_options.single_mut() else {
        return;
    };

    match *active {
        ActiveInputDevice::Gamepad(_) | ActiveInputDevice::SteamInputPad => {
            cursor.visible = false;
            // Start the virtual cursor at screen center so the player doesn't have to hunt for it.
            if let Ok(window) = windows.single() {
                virtual_cursor.screen_pos = Vec2::new(window.width() * 0.5, window.height() * 0.5);
            }
        }
        ActiveInputDevice::MouseKeyboard => {
            cursor.visible = true;
        }
    }

    mouse.clear();
    mouse_state.left_consumed = false;
}

/// Pauses the game when the active — or most recently active — controller
/// disconnects mid-match, and falls back to mouse/keyboard so the pause menu
/// stays usable. Matching [`LastActiveGamepad`] too covers the pad dying right
/// after the player touched the mouse (which resets `ActiveInputDevice`), while
/// an idle bystander pad unplugging still never pauses.
///
/// On disconnect Bevy removes the `Gamepad` component (leaving the entity alive),
/// so every gamepad reader would early-return on the now-dead handle and the OS
/// cursor would stay hidden. Resetting `ActiveInputDevice` is unconditional
/// robustness; only the pause itself is behind the config flag, routed through
/// the shared `RequestGamePauseMessage` consumer so co-op pauses both peers.
pub(crate) fn pause_on_controller_unplug(
    mut events: MessageReader<GamepadConnectionEvent>,
    mut active: ResMut<ActiveInputDevice>,
    last_gamepad: Res<LastActiveGamepad>,
    config: Res<GameConfig>,
    mut pause_writer: MessageWriter<RequestGamePauseMessage>,
) {
    let active_entity = active.gamepad_entity();
    let mut active_lost = false;
    let mut known_pad_lost = false;
    for event in events.read() {
        if !event.disconnected() {
            continue;
        }
        if Some(event.gamepad) == active_entity {
            active_lost = true;
        }
        if Some(event.gamepad) == active_entity || Some(event.gamepad) == last_gamepad.0 {
            known_pad_lost = true;
        }
    }
    if active_lost {
        *active = ActiveInputDevice::MouseKeyboard;
    }
    if known_pad_lost && config.pause_on_controller_disconnect {
        pause_writer.write(RequestGamePauseMessage);
    }
}
