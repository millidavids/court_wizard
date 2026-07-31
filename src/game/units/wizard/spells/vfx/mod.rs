//! Shared visual effects for spells.
//!
//! Provides glow halos, smoke trails, impact spark particles, and sparkle
//! trails that any spell can spawn as sibling entities.

pub(crate) mod area_effects;
pub(crate) mod cast_effects;
pub(in crate::game) mod channel;
pub(crate) mod components;
pub(crate) mod constants;
pub(crate) mod explosion_effects;
pub(crate) mod fire_effects;
pub(crate) mod fire_material;
mod plugin;
pub(crate) mod systems;
pub(crate) mod unit_motes;

pub(super) use plugin::VfxPlugin;
