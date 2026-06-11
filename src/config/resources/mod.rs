mod config_file;
mod game_config;
mod wizard_type;

// Re-export everything to preserve the original public surface at crate::config::resources::*

pub(crate) use config_file::{
    AudioConfig, ConfigFile, SaveDebounceTimer, SavedWindowedGeometry, WindowConfig,
};
pub use config_file::{DisplayMode, VsyncMode};

pub use game_config::{ActiveSave, ColorblindType, ControllerGlyphStyle, GameConfig};
pub(crate) use game_config::{deserialize_action_bar, serialize_action_bar};

pub use wizard_type::WizardType;
