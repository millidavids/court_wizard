//! Disintegrate spell module.
//!
//! Handles disintegrate beam spell that damages enemies in a continuous line.

mod components;
pub mod constants;
mod plugin;
mod systems;

pub use plugin::DisintegratePlugin;
