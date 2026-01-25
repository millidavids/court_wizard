//! Guardian Circle spell module.
//!
//! Handles defensive spell that grants temporary hit points to units in an area.

mod components;
pub mod constants;
mod plugin;
mod styles;
mod systems;

pub use plugin::GuardianCirclePlugin;
