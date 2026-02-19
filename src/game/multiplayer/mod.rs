//! Multiplayer game module.
//!
//! Contains all multiplayer-specific gameplay logic, completely separate from
//! the single-player game systems. Reuses shared helper functions but registers
//! its own systems under `AppState::MultiplayerGame`.

mod components;
mod plugin;

pub use plugin::MultiplayerGamePlugin;
