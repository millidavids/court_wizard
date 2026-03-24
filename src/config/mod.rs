mod error;
pub(crate) mod messages;
mod plugin;
pub(crate) mod progress;
mod resources;
pub(crate) mod save_data;
pub(crate) mod storage;
mod systems;

// Public API exports - only export what's actually used externally
pub use messages::ConfigChanged;
pub use plugin::ConfigPlugin;
pub use resources::{ActiveSave, DisplayMode, GameConfig, VsyncMode, WizardType};
