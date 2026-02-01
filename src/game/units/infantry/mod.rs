//! Infantry plugin module.
//!
//! Handles infantry units on both teams (defenders and attackers).

pub(in crate::game) mod components;
mod plugin;
mod styles;
mod systems;

pub use plugin::InfantryPlugin;
