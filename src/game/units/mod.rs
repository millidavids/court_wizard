//! Units plugin module.
//!
//! Contains all game unit types: wizard and infantry.

pub mod components;
mod constants;
pub mod infantry;
mod systems;
pub mod wizard;

mod plugin;

pub use plugin::UnitsPlugin;
