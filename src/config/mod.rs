mod error;
mod plugin;
pub(crate) mod progress;
mod resources;
mod storage;
mod systems;

// Public API exports - only export what's actually used externally
pub use plugin::ConfigPlugin;
pub use resources::{ConfigChanged, Difficulty, GameConfig, VsyncMode};
