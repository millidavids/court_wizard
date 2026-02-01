//! Disintegrate spell module.
//!
//! Handles disintegrate beam spell that damages enemies in a continuous line.

mod components;
pub(in crate::game::units::wizard) mod constants;
mod plugin;
mod systems;

pub(super) use plugin::DisintegratePlugin;
