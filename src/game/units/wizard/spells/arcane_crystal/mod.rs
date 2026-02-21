//! Arcane Crystal spell module.
//!
//! Places a magical crystal that absorbs incoming spells and re-emits smaller
//! versions at nearby enemies.

pub(crate) mod components;
pub(in crate::game::units::wizard) mod constants;
mod plugin;
mod systems;

pub(super) use plugin::ArcaneCrystalPlugin;
