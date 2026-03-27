pub(in crate::game) mod components;
pub(crate) mod constants;
mod plugin;
pub(in crate::game) mod resources;
pub(in crate::game) mod systems;

pub use components::Aerialist;
pub use plugin::AerialistPlugin;
pub use resources::AerialistAssets;
