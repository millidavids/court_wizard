mod components;
mod constants;
mod fire;
pub(crate) mod messages;
mod plugin;
pub(crate) mod resources;
mod state;
mod systems;

pub use components::GunType;
pub(in crate::game) use plugin::GunslingerPlugin;
pub use resources::GunState;
