//! Units plugin module.
//!
//! Contains all game unit types: wizard and infantry.

pub mod components;
pub mod constants;
pub mod infantry;
mod systems;
pub mod wizard;

mod plugin;

pub use plugin::UnitsPlugin;
