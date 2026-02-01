//! Magic missile spell module.
//!
//! Handles magic missile projectiles that home in on attackers.

mod components;
pub(in crate::game::units::wizard) mod constants;
mod plugin;
mod styles;
mod systems;

pub(super) use plugin::MagicMissilePlugin;
