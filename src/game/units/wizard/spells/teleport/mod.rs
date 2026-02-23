//! Teleport spell module.
//!
//! Two-phase spell that places a destination circle, then teleports all units
//! from a source circle to the destination.

pub(crate) mod components;
pub(crate) mod constants;
mod plugin;
pub(crate) mod systems;

pub(super) use plugin::TeleportPlugin;
