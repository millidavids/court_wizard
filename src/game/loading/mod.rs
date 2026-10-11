pub(in crate::game) mod constants;
pub(crate) mod init;
mod plugin;
pub(crate) mod queue;
pub(in crate::game) mod resources;
pub(in crate::game) mod spawn_queue;
mod systems;
pub(in crate::game) mod terrain_generation;
#[cfg(test)]
mod terrain_seed_compat;
pub(in crate::game) mod upgrade_selection;
pub(in crate::game) mod upgrade_systems;

pub use plugin::LoadingPlugin;
