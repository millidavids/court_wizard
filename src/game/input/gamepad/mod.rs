//! Gamepad input translation layer.
//!
//! Bridges raw gamepad input into the same messages the existing mouse/keyboard
//! pipeline emits, so downstream gameplay and UI systems are source-agnostic.

mod action_translation;
pub(crate) mod arbiter;
pub(crate) mod constants;
pub(crate) mod messages;
mod plugin;
pub(crate) mod resources;
mod rumble;
mod run_conditions;
pub(crate) use run_conditions::gamepad_active;
pub(crate) mod systems;

pub(crate) use plugin::GamepadInputPlugin;
