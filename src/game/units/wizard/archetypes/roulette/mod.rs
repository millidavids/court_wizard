//! Roulette wheel system for the Randomancer wizard archetype.
//!
//! When the player presses spacebar, the roulette wheel spins and
//! randomly selects a spell with high empowerment.

pub(crate) mod constants;
pub(crate) mod messages;
mod plugin;
pub(crate) mod resources;
pub(in crate::game) mod systems;

pub use plugin::RoulettePlugin;
