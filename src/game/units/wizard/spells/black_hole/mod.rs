//! Black Hole spell module.

pub(crate) mod casting;
pub(crate) mod components;
pub(crate) mod constants;
pub(crate) mod gravity;
mod plugin;
pub(crate) mod systems;

pub(super) use plugin::BlackHolePlugin;
