//! Gamepad input messages.
//!
//! These messages bridge gamepad button presses into the existing UI and
//! gameplay pipelines. The input-translation layer emits these; downstream
//! systems (menus, focus navigation, etc.) consume them alongside or instead
//! of the older keyboard-only handlers.

use bevy::prelude::*;
use std::time::Duration;

/// Requests a rumble pulse on the active **Steam Input** controller. Written by
/// the gamepad rumble systems when a Steam Input pad is active (gilrs's own
/// `GamepadRumbleRequest` can't reach a Steam-captured pad); consumed by
/// `steam::input` which drives the `TriggerVibration` FFI. Registered always so
/// the writer is valid even without Steam; only consumed when Steam is running.
#[derive(Message, Debug, Clone, Copy)]
pub(crate) struct ControllerRumbleMessage {
    /// Strong (low-frequency) motor intensity, 0..=1.
    pub strong: f32,
    /// Weak (high-frequency) motor intensity, 0..=1.
    pub weak: f32,
    /// How long the pulse lasts before it is stopped.
    pub duration: Duration,
}

/// Emitted when the user requests a "back" action via the gamepad
/// (East face button / B / Circle, or the Start button as fallback).
///
/// Escape-handling screens should read both `KeyCode::Escape` and this
/// message to support keyboard and gamepad interchangeably.
#[derive(Message, Debug, Clone, Copy)]
pub struct MenuBackPressed;

/// Emitted when the user presses the universal "confirm" gamepad button
/// (South face button / A / Cross) in a UI context. The focus-navigation
/// system translates this into `MouseClicked { button: focused_entity }`.
#[derive(Message, Debug, Clone, Copy)]
pub struct GamepadConfirmPressed;
