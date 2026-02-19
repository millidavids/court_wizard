//! Meteor Fall spell module.
//!
//! A concentration spell that rains meteor projectiles down on a targeted area,
//! dealing fire damage on impact and leaving persistent burning ground hazards.

pub(crate) mod components;
pub(in crate::game::units::wizard) mod constants;
mod plugin;
mod systems;

pub use plugin::MeteorFallPlugin;
