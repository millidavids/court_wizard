mod components;
mod constants;
mod plugin;
pub(crate) mod saves;
pub(super) mod screen;
mod systems;

pub use plugin::GameOverPlugin;
pub(crate) use systems::accumulate_mode_level_stats;
