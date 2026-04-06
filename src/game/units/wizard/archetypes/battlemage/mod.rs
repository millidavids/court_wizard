mod components;
mod constants;
pub(crate) mod messages;
mod plugin;
mod resources;
pub(crate) mod systems;

pub(crate) use constants::CLOSE_CALL_DISTANCE;
pub(in crate::game) use plugin::BattlemagePlugin;
