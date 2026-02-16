pub(super) mod components;
pub(crate) mod constants;
mod plugin;
pub(in crate::game) mod resources;
mod styles;
pub(in crate::game) mod systems;

pub use components::Archer;
pub use plugin::ArcherPlugin;
pub use resources::ArcherAssets;
