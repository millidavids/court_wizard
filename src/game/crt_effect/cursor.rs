//! Barrel-distortion-corrected cursor. One position feeds both world aiming
//! (`CorrectedCursorPosition`) and Bevy picking (`PointerInput`), so UI hover,
//! clicks and spell targeting all land on what the player sees.

use bevy::camera::{NormalizedRenderTarget, RenderTarget};
use bevy::input::ButtonState;
use bevy::picking::pointer::{
    Location, PointerAction, PointerButton, PointerId, PointerInput, PointerLocation,
    PointerPressState,
};
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowEvent, WindowRef};

use super::components::CrtEffectSettings;
use crate::game::input::gamepad::resources::ActiveInputDevice;

/// Barrel-corrected cursor position in logical window coordinates.
///
/// Read this instead of `window.cursor_position()`. The gamepad virtual cursor
/// overwrites it during gameplay.
#[derive(Resource, Default)]
pub(crate) struct CorrectedCursorPosition(pub Option<Vec2>);

/// Maps a raw cursor position to the content drawn under it.
///
/// The CRT shader shows `barrel_distort(uv)` at screen `uv`, so applying the
/// same forward distortion to the cursor targets what the player sees. Identity
/// when the barrel effect is off.
pub(crate) fn barrel_corrected_position(
    raw: Vec2,
    window_size: Vec2,
    crt: Option<&CrtEffectSettings>,
) -> Vec2 {
    let Some(crt) = crt.filter(|crt| crt.is_barrel_active()) else {
        return raw;
    };
    let viewport_min = Vec2::new(crt.viewport_x, crt.viewport_y) * window_size;
    let viewport_size = Vec2::new(crt.viewport_w, crt.viewport_h) * window_size;
    if viewport_size.cmple(Vec2::ZERO).any() {
        return raw;
    }
    let centered = (raw - viewport_min) / viewport_size - 0.5;
    let distorted = centered * (1.0 + crt.barrel_distortion * centered.length_squared()) + 0.5;
    // Distortion pushes edge positions outside the viewport.
    viewport_min + (distorted * viewport_size).clamp(Vec2::ZERO, viewport_size)
}

/// Mouse pointer state carried between frames.
#[derive(Default)]
pub(crate) struct MousePointer {
    /// Last raw OS cursor position.
    raw: Option<Vec2>,
    /// Last position sent to picking; `None` while parked.
    emitted: Option<Vec2>,
}

impl MousePointer {
    fn move_to(
        &mut self,
        to: Vec2,
        target: &NormalizedRenderTarget,
        inputs: &mut MessageWriter<PointerInput>,
    ) {
        if self.emitted == Some(to) {
            return;
        }
        let delta = self.emitted.map_or(Vec2::ZERO, |from| to - from);
        self.emitted = Some(to);
        self.send(PointerAction::Move { delta }, target, inputs);
    }

    fn send(
        &self,
        action: PointerAction,
        target: &NormalizedRenderTarget,
        inputs: &mut MessageWriter<PointerInput>,
    ) {
        if let Some(position) = self.emitted {
            let location = Location {
                target: target.clone(),
                position,
            };
            inputs.write(PointerInput::new(PointerId::Mouse, location, action));
        }
    }
}

/// Drives Bevy's mouse pointer from the barrel-corrected cursor (Bevy's own
/// mouse input is disabled) and publishes the same position as
/// [`CorrectedCursorPosition`].
///
/// Only the left button reaches picking: UI buttons don't distinguish buttons,
/// and right-click casts read `ButtonInput` directly.
///
/// While a controller is active the pointer is parked (no location), which
/// un-hovers everything. A mouse button or wheel event wakes it at once; plain
/// motion waits for the device switch so a bumped mouse doesn't steal focus.
/// A held button keeps it awake: the pad stays active through a click while
/// it's being held (`detect_active_input_device`), and parking would drop it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn drive_mouse_pointer(
    mut window_events: MessageReader<WindowEvent>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    crt: Query<&CrtEffectSettings>,
    active_device: Res<ActiveInputDevice>,
    mut pointer: Local<MousePointer>,
    mut mouse: Query<(&PointerId, &mut PointerLocation, &PointerPressState)>,
    mut corrected: ResMut<CorrectedCursorPosition>,
    mut inputs: MessageWriter<PointerInput>,
) {
    let Ok((window_entity, window)) = windows.single() else {
        return;
    };
    let Some(target) = RenderTarget::Window(WindowRef::Entity(window_entity)).normalize(None)
    else {
        return;
    };
    let crt = crt.single().ok();
    let correct = |raw: Vec2| barrel_corrected_position(raw, window.size(), crt);
    let mut woken = false;

    for event in window_events.read() {
        let action = match event {
            // Motion is coalesced into one Move per frame, sent below.
            WindowEvent::CursorMoved(moved) if moved.window == window_entity => {
                pointer.raw = Some(moved.position);
                continue;
            }
            WindowEvent::CursorLeft(left) if left.window == window_entity => {
                pointer.raw = None;
                continue;
            }
            WindowEvent::MouseButtonInput(input)
                if input.window == window_entity && input.button == MouseButton::Left =>
            {
                match input.state {
                    ButtonState::Pressed => PointerAction::Press(PointerButton::Primary),
                    ButtonState::Released => PointerAction::Release(PointerButton::Primary),
                }
            }
            WindowEvent::MouseWheel(wheel) if wheel.window == window_entity => {
                PointerAction::Scroll {
                    x: wheel.x,
                    y: wheel.y,
                    unit: wheel.unit,
                    phase: wheel.phase,
                }
            }
            _ => continue,
        };
        // Buttons and the wheel act where the cursor is at that moment, so bring
        // the pointer there first (waking it if parked).
        let Some(raw) = pointer.raw else {
            continue;
        };
        woken = true;
        pointer.move_to(correct(raw), &target, &mut inputs);
        pointer.send(action, &target, &mut inputs);
    }

    let corrected_pos = pointer.raw.map(correct);
    corrected.0 = corrected_pos;

    let held = mouse
        .iter()
        .any(|(id, _, press)| *id == PointerId::Mouse && press.is_primary_pressed());

    // Also re-syncs when the correction changes without motion (CRT toggle,
    // resize, viewport change): `CrtEffectSettings` is written every frame, so
    // change detection can't tell us.
    match corrected_pos.filter(|_| woken || held || !active_device.is_gamepad()) {
        Some(to) => pointer.move_to(to, &target, &mut inputs),
        None if pointer.emitted.is_some() => {
            // Release before dropping the location so picking ends the press.
            if held {
                let release = PointerAction::Release(PointerButton::Primary);
                pointer.send(release, &target, &mut inputs);
            }
            for (id, mut location, _) in &mut mouse {
                if *id == PointerId::Mouse {
                    location.location = None;
                }
            }
            pointer.emitted = None;
        }
        None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: Vec2 = Vec2::new(1600.0, 900.0);

    fn barrel(strength: f32) -> CrtEffectSettings {
        CrtEffectSettings {
            barrel_distortion: strength,
            enabled: 1.0,
            viewport_x: 0.0,
            viewport_y: 0.0,
            viewport_w: 1.0,
            viewport_h: 1.0,
            ..default()
        }
    }

    #[test]
    fn centre_is_unchanged() {
        let centre = WINDOW / 2.0;
        let corrected = barrel_corrected_position(centre, WINDOW, Some(&barrel(0.2)));
        assert!(corrected.abs_diff_eq(centre, 1e-3));
    }

    #[test]
    fn edges_are_pushed_outward() {
        let raw = Vec2::new(1200.0, 450.0);
        let corrected = barrel_corrected_position(raw, WINDOW, Some(&barrel(0.2)));
        assert!(corrected.x > raw.x);
        assert!((corrected.y - raw.y).abs() < 1e-3);
    }

    #[test]
    fn corners_stay_inside_the_viewport() {
        let corrected = barrel_corrected_position(WINDOW, WINDOW, Some(&barrel(0.5)));
        assert!(corrected.abs_diff_eq(WINDOW, 1e-3));
    }

    #[test]
    fn no_barrel_is_identity() {
        let raw = Vec2::new(1500.0, 50.0);
        assert_eq!(barrel_corrected_position(raw, WINDOW, None), raw);
        assert_eq!(
            barrel_corrected_position(raw, WINDOW, Some(&barrel(0.0))),
            raw
        );
    }
}
