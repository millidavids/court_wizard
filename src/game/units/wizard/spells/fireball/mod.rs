//! Fireball spell module.
//!
//! Handles fireball projectiles that explode on impact.

pub(crate) mod casting;
pub(crate) mod components;
pub(crate) mod constants;
mod plugin;
pub(crate) mod projectile;
pub(crate) mod systems;

pub(super) use plugin::FireballPlugin;
