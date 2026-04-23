mod error;
pub(crate) mod input_bindings;
pub(crate) mod messages;
mod plugin;
pub(crate) mod progress;
mod resources;
pub(crate) mod save_data;
pub(crate) mod storage;
mod systems;

// Public API exports - only export what's actually used externally
pub(crate) use input_bindings::InputBindings;
pub use messages::ConfigChanged;
pub use plugin::ConfigPlugin;
pub(crate) use resources::SavedWindowedGeometry;
pub use resources::{
    ActiveSave, ColorblindType, ControllerGlyphStyle, DisplayMode, GameConfig, VsyncMode,
    WizardType,
};
