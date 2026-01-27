//! Input handling for the game.
//!
//! Centralizes all input detection to avoid duplicate queries.
//! Input systems send events that other game systems consume.

pub mod components;
pub mod events;
mod plugin;
mod systems;

pub use components::MouseButtonState;
pub use plugin::InputPlugin;
