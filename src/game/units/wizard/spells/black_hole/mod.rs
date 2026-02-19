//! Black Hole spell module.

pub(in crate::game::units::wizard::spells) mod components;
pub(in crate::game::units::wizard) mod constants;
mod plugin;
mod systems;

pub(super) use plugin::BlackHolePlugin;
