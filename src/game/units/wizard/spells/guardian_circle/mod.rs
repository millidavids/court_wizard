//! Guardian Circle spell module.
//!
//! Handles defensive spell that grants temporary hit points to units in an area.

pub(crate) mod components;
pub(crate) mod constants;
mod plugin;
mod styles;
pub(crate) mod systems;

pub(super) use plugin::GuardianCirclePlugin;
