mod components;
mod constants;
mod plugin;
mod systems;

pub use plugin::CauldronMenuPlugin;

// Re-exports for tutorial system
pub(crate) use components::CauldronMenuButtonAction;
