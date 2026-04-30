pub(super) mod charge;
pub(super) mod combat;
pub(in crate::game) mod components;
pub(in crate::game) mod constants;
mod plugin;
pub(in crate::game) mod resources;
pub(in crate::game) mod systems;

pub(crate) use components::MeleeDamageReduction;
pub(super) use plugin::OgrePlugin;
