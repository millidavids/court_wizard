//! Guardian Circle spell module.
//!
//! Handles defensive spell that grants temporary hit points to units in an area.

mod components;
pub(in crate::game::units::wizard) mod constants;
mod plugin;
mod styles;
mod systems;

pub(super) use plugin::GuardianCirclePlugin;
