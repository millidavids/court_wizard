//! Fireball spell module.
//!
//! Handles fireball projectiles that explode on impact.

pub(in crate::game::units::wizard::spells) mod components;
pub(in crate::game::units::wizard) mod constants;
mod plugin;
mod styles;
mod systems;

pub(super) use plugin::FireballPlugin;
