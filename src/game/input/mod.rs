//! Input handling for the game.
//!
//! Centralizes all input detection to avoid duplicate queries.
//! Input systems send messages that other game systems consume.

pub(crate) mod components;
pub(crate) mod messages;
mod plugin;
pub(crate) mod run_conditions;
mod systems;

pub(crate) use components::MouseButtonState;
pub use plugin::InputPlugin;
