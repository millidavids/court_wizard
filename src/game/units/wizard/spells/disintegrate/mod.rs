//! Disintegrate spell module.
//!
//! Handles disintegrate beam spell that damages enemies in a continuous line.

pub(crate) mod components;
pub(crate) mod constants;
mod plugin;
pub(crate) mod systems;

pub(super) use plugin::DisintegratePlugin;
