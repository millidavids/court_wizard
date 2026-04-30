pub(super) mod beams;
pub(in crate::game) mod components;
pub(in crate::game) mod constants;
pub(super) mod movement;
mod plugin;
pub(in crate::game) mod resources;
pub(in crate::game) mod systems;

pub(crate) use components::Ray;
pub(crate) use components::RayEye;
pub(crate) use components::RayEyeDying;
pub(crate) use components::RayEyeType;
pub(super) use plugin::RayPlugin;
