//! Units plugin module.
//!
//! Contains all game unit types: wizard, infantry, and archers.

pub(crate) mod archer;
pub(crate) mod components;
pub(crate) mod constants;
pub(super) mod infantry;
pub(super) mod king;
mod systems;
pub(crate) mod wizard;

mod plugin;

pub use plugin::UnitsPlugin;
