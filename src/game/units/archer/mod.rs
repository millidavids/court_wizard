pub(crate) mod arrows;
pub(crate) mod combat;
pub(in crate::game) mod components;
pub(crate) mod constants;
pub(crate) mod movement;
mod plugin;
pub(in crate::game) mod resources;
pub(in crate::game) mod systems;

pub use components::Archer;
pub use plugin::ArcherPlugin;
pub use resources::ArcherAssets;
