mod components;
mod constants;
pub(super) mod interaction;
mod plugin;
pub(super) mod setup;
mod systems;

pub use plugin::CauldronMenuPlugin;

// Re-exports for tutorial system
pub(crate) use components::CauldronMenuButtonAction;
