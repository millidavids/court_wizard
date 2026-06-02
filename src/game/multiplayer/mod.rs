//! Multiplayer game module.
//!
//! Contains all multiplayer-specific gameplay logic, completely separate from
//! the single-player game systems. Reuses shared helper functions but registers
//! its own systems under `AppState::MultiplayerGame`.

pub(crate) mod components;
pub(in crate::game) mod crdt_sync;
pub(in crate::game) mod guest_snapshot;
pub(in crate::game) mod guest_systems;
pub(in crate::game) mod guest_visuals;
pub(in crate::game) mod host_systems;
pub(crate) mod loading;
mod plugin;
pub(in crate::game) mod score_stats;
pub(crate) mod spawning;
pub(in crate::game) mod spell_sync;
mod systems;

pub use plugin::MultiplayerGamePlugin;
