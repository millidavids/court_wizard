//! Units plugin module.
//!
//! Contains all game unit types: wizard, defenders, and attackers.

pub mod attacker;
pub mod components;
pub mod defender;
pub mod wizard;

mod plugin;

pub use plugin::UnitsPlugin;
