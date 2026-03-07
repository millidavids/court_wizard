mod components;
mod constants;
pub mod messages;
mod plugin;
mod resources;
mod systems;

pub(in crate::game) use plugin::BattlemagePlugin;
pub(crate) use constants::CLOSE_CALL_DISTANCE;
