pub(crate) mod achievement_id;
mod error;
pub(crate) mod input_bindings;
pub(crate) mod messages;
mod plugin;
pub(crate) mod progress;
mod resource_paths;
mod resources;
pub(crate) mod save_data;
pub(crate) mod save_encoding;
pub(crate) mod storage;
mod systems;

// Public API exports - only export what's actually used externally
pub(crate) use input_bindings::InputBindings;
pub use messages::ConfigChanged;
pub use plugin::{ConfigPlugin, PreExitCleanupSet};
pub(crate) use resource_paths::resource_root;
pub(crate) use resources::SavedWindowedGeometry;
pub use resources::{
    ActiveSave, ColorblindType, ControllerGlyphStyle, DisplayMode, GameConfig, VsyncMode,
    WizardType,
};
