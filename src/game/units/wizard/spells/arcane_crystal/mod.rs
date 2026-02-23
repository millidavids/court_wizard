//! Arcane Crystal spell module.
//!
//! Places a magical crystal that absorbs incoming spells and re-emits smaller
//! versions at nearby enemies.

pub(crate) mod components;
pub(crate) mod constants;
mod plugin;
pub(crate) mod systems;

pub(super) use plugin::ArcaneCrystalPlugin;
