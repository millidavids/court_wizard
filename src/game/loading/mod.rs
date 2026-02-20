mod constants;
mod plugin;
pub(in crate::game) mod resources;
pub(in crate::game) mod spawn_queue;
mod systems;
mod upgrade_selection;
mod upgrade_systems;

pub use plugin::LoadingPlugin;
