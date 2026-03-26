pub(in crate::game) mod components;
pub(in crate::game) mod constants;
mod plugin;
pub(in crate::game) mod resources;
pub(in crate::game) mod systems;

pub(super) use plugin::OgrePlugin;
pub(crate) use components::MeleeDamageReduction;
