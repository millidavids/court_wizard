mod error;
mod plugin;
mod resources;
mod storage;
mod systems;

// Public API exports - some may be unused in main.rs but are available for library users
#[allow(unused_imports)]
pub use error::{ConfigError, ConfigResult};
pub use plugin::ConfigPlugin;
#[allow(unused_imports)]
pub use resources::{
    AudioConfig, ConfigChanged, ConfigFile, Difficulty, GameConfig, SaveConfigEvent,
    SaveDebounceTimer, VsyncMode, WindowConfig,
};
