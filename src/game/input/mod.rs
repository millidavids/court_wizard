//! Input handling for the game.
//!
//! Centralizes all input detection to avoid duplicate queries.
//! Input systems send events that other game systems consume.

pub mod events;
mod plugin;
mod systems;

pub use plugin::InputPlugin;
