//! Fireball spell module.
//!
//! Handles fireball projectiles that explode on impact.

pub(crate) mod components;
pub(crate) mod constants;
mod plugin;
mod styles;
pub(crate) mod systems;

pub(super) use plugin::FireballPlugin;
