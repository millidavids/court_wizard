//! Squall spell module.
//!
//! A concentration spell that rains ice projectiles down on a targeted area,
//! dealing frost damage and slowing enemy movement.

pub(crate) mod casting;
pub(crate) mod components;
pub(crate) mod constants;
mod plugin;
pub(crate) mod shards;
pub(crate) mod systems;

pub use plugin::SquallPlugin;
