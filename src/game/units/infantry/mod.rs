//! Infantry plugin module.
//!
//! Handles infantry units on both teams (defenders and attackers).

pub(in crate::game) mod components;
pub(in crate::game) mod constants;
mod plugin;
pub(in crate::game) mod resources;
pub(in crate::game) mod systems;

pub use components::Infantry;
pub use plugin::InfantryPlugin;
