use bevy::prelude::*;
use std::path::PathBuf;

use super::resources::*;
use super::systems::*;

/// Configuration plugin for managing game settings.
///
/// This plugin provides a complete configuration system that:
/// - Loads configuration from a TOML file at startup
/// - Applies window settings to Bevy's `Window` component
/// - Provides a `GameConfig` resource that can be modified at runtime
/// - Automatically persists changes to disk when window is resized or `GameConfig` changes
///
/// # Architecture
///
/// The plugin uses a three-layer architecture:
/// 1. **ConfigFile** - TOML serialization layer (disk ↔ memory)
/// 2. **Bevy Components** - Runtime source of truth for window/audio settings
/// 3. **GameConfig Resource** - Runtime source of truth for game-specific settings
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use the_game::config::ConfigPlugin;
///
/// fn main() {
///     App::new()
///         .add_plugins(ConfigPlugin::default()) // Uses "config.toml"
///         .run();
/// }
/// ```
///
/// Using a custom config path:
///
/// ```
/// use bevy::prelude::*;
/// use the_game::config::ConfigPlugin;
/// use std::path::PathBuf;
///
/// fn main() {
///     App::new()
///         .add_plugins(ConfigPlugin {
///             config_path: PathBuf::from("settings/game.toml"),
///         })
///         .run();
/// }
/// ```
#[allow(clippy::needless_doctest_main)]
pub struct ConfigPlugin {
    /// Path to the configuration file (relative or absolute)
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
