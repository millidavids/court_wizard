//! Units plugin module.
//!
//! Contains all game unit types: wizard, infantry, and archers.

pub mod archer;
pub mod components;
pub mod constants;
pub mod infantry;
pub mod king;
mod systems;
pub mod wizard;

mod plugin;

pub use plugin::UnitsPlugin;
