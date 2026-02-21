//! Magic missile spell module.
//!
//! Handles magic missile projectiles that home in on attackers.

pub(crate) mod components;
pub(crate) mod constants;
mod plugin;
mod styles;
pub(crate) mod systems;

pub(super) use plugin::MagicMissilePlugin;
