//! Magic missile spell module.
//!
//! Handles magic missile projectiles that home in on attackers.

pub(crate) mod casting;
pub(crate) mod components;
pub(crate) mod constants;
pub(crate) mod missile;
mod plugin;
pub(crate) mod systems;

pub(super) use plugin::MagicMissilePlugin;
