pub(super) mod ai;
pub(in crate::game) mod components;
pub(in crate::game) mod constants;
mod plugin;
pub(in crate::game) mod resources;
pub(super) mod spells;
pub(in crate::game) mod systems;

pub(crate) use components::DarkMage;
pub(super) use plugin::DarkMagePlugin;
