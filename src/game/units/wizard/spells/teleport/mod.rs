//! Teleport spell module.
//!
//! Two-phase spell that places a destination circle, then teleports all units
//! from a source circle to the destination.

pub mod components;
pub mod constants;
mod plugin;
mod systems;

pub use plugin::TeleportPlugin;
