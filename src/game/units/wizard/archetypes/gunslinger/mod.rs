mod components;
mod constants;
pub(crate) mod messages;
mod plugin;
mod resources;
mod systems;

pub use components::GunType;
pub(in crate::game) use plugin::GunslingerPlugin;
pub use resources::GunState;
