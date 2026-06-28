//! gilrs → [`GamepadActionState`] fallback producer.
//!
//! When no Steam Input controller is driving (the game is launched outside Steam,
//! or Steam Input is disabled for the pad), this fills the shared action state
//! from Bevy's raw `Gamepad` components using the default button layout — so every
//! downstream system reads the same `GamepadActionState` regardless of source.
//!
//! Only one producer writes the state per frame (this is gated on a gilrs pad
//! being the active device; the Steam poll is gated on a Steam handle). Edge state
//! lives in `state.buttons` (a `ButtonInput`): we `clear()` the per-frame deltas
//! and then `press`/`release` from the current button state, leaving the held-set
//! intact across frames and producer hand-offs so `just_pressed` stays correct.

use bevy::prelude::*;

use super::super::resources::{ActiveInputDevice, GamepadAimSettings};
use crate::game::input::action_state::{ControllerKind, GamepadAction, GamepadActionState};

/// Vendor IDs for the controller families we ship glyphs for.
fn kind_from_vendor(vendor: Option<u16>) -> ControllerKind {
    match vendor {
        Some(0x054C) => ControllerKind::PlayStation, // Sony
        Some(0x057E) => ControllerKind::Switch,      // Nintendo
        Some(0x28DE) => ControllerKind::SteamDeck,   // Valve
        _ => ControllerKind::Xbox,
    }
}

/// Fills `GamepadActionState` from the active gilrs pad. No-op when the active
/// device is mouse/keyboard or a Steam Input pad (the Steam poll owns that case).
pub(crate) fn fill_action_state_from_gilrs(
    active: Res<ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    aim: Res<GamepadAimSettings>,
    mut state: ResMut<GamepadActionState>,
) {
    let entity = match *active {
        ActiveInputDevice::Gamepad(e) => e,
        // The Steam Input poll producer owns the action state this frame.
        ActiveInputDevice::SteamInputPad => return,
        // No controller: clear so stale gamepad state can't leak to consumers
        // that aren't gated on `is_gamepad()`.
        ActiveInputDevice::MouseKeyboard => {
            state.clear_inputs();
            return;
        }
    };
    let Ok(gamepad) = gamepads.get(entity) else {
        return;
    };

    state.active_gilrs_entity = Some(entity);
    state.active_steam_handle = None;
    state.controller_kind = kind_from_vendor(gamepad.vendor_id());

    let axis = |a| gamepad.get(a).unwrap_or(0.0);
    state.left_stick = Vec2::new(axis(GamepadAxis::LeftStickX), axis(GamepadAxis::LeftStickY));
    state.right_stick = Vec2::new(
        axis(GamepadAxis::RightStickX),
        axis(GamepadAxis::RightStickY),
    );
    state.left_trigger = gamepad.get(GamepadButton::LeftTrigger2).unwrap_or(0.0);
    state.right_trigger = gamepad.get(GamepadButton::RightTrigger2).unwrap_or(0.0);

    let thresh = aim.trigger_threshold;
    let south = gamepad.pressed(GamepadButton::South);
    // Each face/D-pad button maps to every semantic action it can represent; the
    // consuming systems are context-gated so only the relevant one is read (e.g.
    // South is `Activate` in gameplay, `UIConfirm` in menus, `StudyConfirm` in the
    // study graph — never simultaneously).
    let downs = [
        (GamepadAction::Activate, south),
        (GamepadAction::UIConfirm, south),
        (GamepadAction::StudyConfirm, south),
        (
            GamepadAction::UIBack,
            gamepad.pressed(GamepadButton::East) || gamepad.pressed(GamepadButton::Start),
        ),
        (
            GamepadAction::OpenSpellBook,
            gamepad.pressed(GamepadButton::West),
        ),
        (
            GamepadAction::OpenCauldron,
            gamepad.pressed(GamepadButton::North),
        ),
        (GamepadAction::Pause, gamepad.pressed(GamepadButton::Start)),
        (
            GamepadAction::AbilityUp,
            gamepad.pressed(GamepadButton::DPadUp),
        ),
        (
            GamepadAction::AbilityDown,
            gamepad.pressed(GamepadButton::DPadDown),
        ),
        (
            GamepadAction::AbilityLeft,
            gamepad.pressed(GamepadButton::DPadLeft),
        ),
        (
            GamepadAction::AbilityRight,
            gamepad.pressed(GamepadButton::DPadRight),
        ),
        (
            GamepadAction::TabNext,
            gamepad.pressed(GamepadButton::RightTrigger),
        ),
        (
            GamepadAction::TabPrev,
            gamepad.pressed(GamepadButton::LeftTrigger),
        ),
        (GamepadAction::PrimaryCast, state.right_trigger >= thresh),
        (GamepadAction::SecondaryCast, state.left_trigger >= thresh),
    ];

    state.buttons.clear();
    for (action, down) in downs {
        if down {
            state.buttons.press(action);
        } else {
            state.buttons.release(action);
        }
    }
}
