//! Spells plugin module.
//!
//! Handles wizard spells, projectiles, and spell effects.

pub mod components;
pub mod disintegrate;
pub mod magic_missile;
mod plugin;
mod systems;

pub use plugin::SpellsPlugin;
