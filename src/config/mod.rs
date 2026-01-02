mod defaults;
mod error;
mod plugin;
mod resources;
mod systems;

#[cfg(test)]
mod tests;

// Public API exports - some may be unused in main.rs but are available for library users
#[allow(unused_imports)]
pub use error::{ConfigError, ConfigResult};
pub use plugin::ConfigPlugin;
#[allow(unused_imports)]
pub use resources::{ConfigFile, Difficulty, GameConfig, VsyncMode, WindowMode};
