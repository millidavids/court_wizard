pub(crate) mod components;
pub(crate) mod constants;
mod plugin;
pub(in crate::game) mod resources;
pub(in crate::game) mod systems;

pub(super) use plugin::LichPlugin;
pub(crate) use components::Lich;
