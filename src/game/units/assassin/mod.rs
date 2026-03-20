pub(in crate::game) mod components;
pub(crate) mod constants;
mod plugin;
pub(in crate::game) mod resources;
pub(in crate::game) mod systems;

pub use components::Assassin;
pub use plugin::AssassinPlugin;
pub use resources::AssassinAssets;
