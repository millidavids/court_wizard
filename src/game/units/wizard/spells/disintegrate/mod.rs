//! Disintegrate spell module.
//!
//! Handles disintegrate beam spell that damages enemies in a continuous line.

pub(in crate::game::units::wizard::spells) mod components;
pub(in crate::game::units::wizard) mod constants;
mod plugin;
mod systems;

pub(super) use plugin::DisintegratePlugin;
