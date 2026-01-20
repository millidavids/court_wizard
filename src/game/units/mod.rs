//! Units plugin module.
//!
//! Contains all game unit types: wizard and infantry.

pub mod components;
pub mod infantry;
pub mod wizard;

mod plugin;

pub use plugin::UnitsPlugin;
