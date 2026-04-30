//! Meteor Fall spell module.
//!
//! A concentration spell that rains meteor projectiles down on a targeted area,
//! dealing fire damage on impact and leaving persistent burning ground hazards.

pub(crate) mod casting;
pub(crate) mod components;
pub(crate) mod constants;
pub(crate) mod meteor;
mod plugin;
pub(crate) mod systems;

pub use plugin::MeteorFallPlugin;
