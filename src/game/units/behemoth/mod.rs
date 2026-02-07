mod components;
mod constants;
pub(super) mod messages;
mod plugin;
pub(in crate::game) mod resources;
pub(in crate::game) mod systems;

pub(super) use plugin::BehemothPlugin;
