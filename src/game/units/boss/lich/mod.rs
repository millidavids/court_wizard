pub(super) mod combat;
pub(crate) mod components;
pub(crate) mod constants;
mod plugin;
pub(in crate::game) mod resources;
pub(super) mod spawn;
pub(in crate::game) mod systems;

pub(crate) use components::Lich;
pub(super) use plugin::LichPlugin;
