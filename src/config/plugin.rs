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
/// - Persists changes to disk when `SaveConfigEvent` is sent
///
/// # Architecture
///
/// The plugin uses a three-layer architecture:
/// 1. **ConfigFile** - TOML serialization layer (disk ↔ memory)
/// 2. **Bevy Components** - Runtime source of truth for window/audio settings
/// 3. **GameConfig Resource** - Runtime source of truth for game-specific settings
///
/// # Manual Saving
///
/// Configuration changes are NOT automatically saved. To persist changes to disk,
/// send a `SaveConfigEvent`:
///
/// ```
/// use bevy::prelude::*;
/// use the_game::config::SaveConfigEvent;
///
/// fn save_settings(mut save_events: EventWriter<SaveConfigEvent>) {
///     save_events.send(SaveConfigEvent);
/// }
/// ```
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
        // Insert resources
        app.insert_resource(ConfigPath(self.config_path.clone()));
        app.init_resource::<super::resources::SaveDebounceTimer>();

        // Add message for manual config saving
        app.add_message::<super::resources::SaveConfigEvent>();

        // Add systems
        app.add_systems(Startup, load_and_apply_config);
        app.add_systems(
            Update,
            (
                mark_save_pending_on_resize,
                save_config_on_debounce_timer,
                save_config_on_event,
            ),
        );
    }

    fn name(&self) -> &str {
        "ConfigPlugin"
    }
}
