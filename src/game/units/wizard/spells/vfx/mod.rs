//! Shared visual effects for spells.
//!
//! Provides glow halos, smoke trails, impact spark particles, and sparkle
//! trails that any spell can spawn as sibling entities.

pub(crate) mod components;
pub(crate) mod constants;
mod plugin;
pub(crate) mod systems;

pub(super) use plugin::VfxPlugin;
