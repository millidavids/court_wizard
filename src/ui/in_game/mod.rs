//! In-game UI and input systems.

mod components;
mod constants;
pub(super) mod plugin;
mod systems;

// Re-exports for tutorial system
pub(crate) use components::{HudButtonAction, KingHealthBarFill, ManaBarFill, WaveDisplay};
