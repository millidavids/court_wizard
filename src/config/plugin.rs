use bevy::prelude::*;

use super::systems::*;

/// Configuration plugin for managing game settings in browser localStorage.
///
/// This plugin provides a complete configuration system that:
/// - Loads configuration from browser localStorage at startup
/// - Applies settings to Bevy components (Window, GameConfig, etc.)
/// - **Bevy components are the single source of truth** (no duplicate state)
/// - Implements unified debouncing for all config changes
/// - Persists changes to localStorage after 0.5s of inactivity
///
/// # Architecture: Single Source of Truth
///
/// ```
/// localStorage (persistent)
///     ↕ (load/save only)
/// ConfigFile (temporary, serialization only)
///     ↕ (apply at startup, build at save)
/// Bevy Components (single source of truth)
///     - Window component (window settings)
///     - GameConfig resource (game settings)
/// ```
///
/// **ConfigFile is NOT a runtime resource.** It only exists briefly during:
/// 1. Startup: Load TOML → apply to Bevy components → discard
/// 2. Save: Read Bevy components → build ConfigFile → serialize → save → discard
///
/// # Debouncing
///
/// All config changes trigger a unified 0.5s debounce timer via the
/// `ConfigChanged` message. Any system can trigger a debounced save:
///
/// ```rust
/// use bevy::prelude::*;
/// use the_game::config::ConfigChanged;
///
/// fn my_system(mut events: MessageWriter<ConfigChanged>) {
///     // ... modify Bevy components ...
///     events.write(ConfigChanged);  // Trigger debounced save
/// }
/// ```
///
/// Built-in triggers:
/// - Window resize events
/// - GameConfig resource changes
///
/// Future triggers can easily be added by sending ConfigChanged.
///
/// After 0.5s of inactivity, current state is saved to localStorage.
///
/// # Manual Save
///
/// Send `SaveConfigEvent` to bypass debounce and save immediately:
///
/// ```rust
/// use bevy::prelude::*;
/// use the_game::config::SaveConfigEvent;
///
/// fn save_on_quit(mut events: MessageWriter<SaveConfigEvent>) {
///     events.write(SaveConfigEvent);
/// }
/// ```
#[allow(clippy::needless_doctest_main)]
#[derive(Default)]
pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        // Insert resources
        app.init_resource::<super::resources::SaveDebounceTimer>();
        // NOTE: ConfigFile is NOT a resource - it's only used for serialization

        // Add messages
        app.add_message::<super::resources::SaveConfigEvent>();
        app.add_message::<super::resources::ConfigChanged>();

        // Add systems
        app.add_systems(Startup, load_and_apply_config);
        app.add_systems(
            Update,
            (
                // Change detection systems (emit ConfigChanged)
                detect_window_resize,
                detect_game_config_changes,
                // Unified debounce trigger
                mark_save_on_config_changed,
                // Save systems
                save_config_on_debounce_timer,
                save_config_on_event,
            ),
        );
    }

    fn name(&self) -> &str {
        "ConfigPlugin"
    }
}
