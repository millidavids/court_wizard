use bevy::prelude::*;
use std::path::PathBuf;

use super::resources::*;
use super::systems::*;

/// Configuration plugin for managing game settings.
///
/// This plugin:
/// - Loads configuration from a TOML file at startup
/// - Provides a GameConfig resource that can be modified at runtime
/// - Automatically applies changes to the window when GameConfig is modified
/// - Persists configuration changes to disk
pub struct ConfigPlugin {
    /// Path to the configuration file
    pub config_path: PathBuf,
}

impl Default for ConfigPlugin {
    fn default() -> Self {
        Self {
            config_path: PathBuf::from("config.toml"),
        }
    }
}

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        // Insert the config path resource
        app.insert_resource(ConfigPath(self.config_path.clone()));

        // Add systems
        app.add_systems(Startup, load_and_apply_config);
        app.add_systems(
            Update,
            (
                persist_window_on_resize,
                persist_game_config_on_change.run_if(resource_changed::<GameConfig>),
            ),
        );
    }

    fn name(&self) -> &str {
        "ConfigPlugin"
    }
}
