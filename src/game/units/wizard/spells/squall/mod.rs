//! Squall spell module.
//!
//! A concentration spell that rains ice projectiles down on a targeted area,
//! dealing frost damage and slowing enemy movement.

pub(crate) mod components;
pub(in crate::game::units::wizard) mod constants;
mod plugin;
mod styles;
mod systems;

pub use plugin::SquallPlugin;
