pub(in crate::game) mod constants;
mod plugin;
pub(in crate::game) mod resources;
pub(in crate::game) mod spawn_queue;
mod systems;
pub(in crate::game) mod upgrade_selection;
pub(in crate::game) mod upgrade_systems;

pub use plugin::LoadingPlugin;
