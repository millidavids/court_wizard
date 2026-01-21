//! Units plugin module.
//!
//! Contains all game unit types: wizard and infantry.

pub mod components;
mod constants;
pub mod infantry;
pub mod wizard;

mod plugin;

pub use plugin::UnitsPlugin;
