//! Infantry plugin module.
//!
//! Handles infantry units on both teams (defenders and attackers).

pub(in crate::game) mod components;
mod plugin;
pub(in crate::game) mod resources;
mod styles;
pub(in crate::game) mod systems;

pub use components::Infantry;
pub use plugin::InfantryPlugin;
