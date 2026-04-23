//! Gamepad input messages.
//!
//! These messages bridge gamepad button presses into the existing UI and
//! gameplay pipelines. The input-translation layer emits these; downstream
//! systems (menus, focus navigation, etc.) consume them alongside or instead
//! of the older keyboard-only handlers.

use bevy::prelude::*;

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
