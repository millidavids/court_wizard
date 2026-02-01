//! Teleport spell module.
//!
//! Two-phase spell that places a destination circle, then teleports all units
//! from a source circle to the destination.

mod components;
pub(in crate::game::units::wizard) mod constants;
mod plugin;
mod systems;

pub(super) use plugin::TeleportPlugin;
