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
/// use court_wizard::config::ConfigChanged;
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
/// Send `SaveConfigMessage` to bypass debounce and save immediately:
///
/// ```rust
/// use bevy::prelude::*;
/// use court_wizard::config::SaveConfigMessage;
///
/// fn save_on_quit(mut events: MessageWriter<SaveConfigMessage>) {
///     events.write(SaveConfigMessage);
/// }
/// ```
#[allow(clippy::needless_doctest_main)]
#[derive(Default)]
pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        // Insert resources
        app.init_resource::<super::resources::SaveDebounceTimer>();
        app.init_resource::<super::resources::ActiveSave>();
        // NOTE: ConfigFile is NOT a resource - it's only used for serialization

        // Add messages
        app.add_message::<super::messages::SaveConfigMessage>();
        app.add_message::<super::messages::ConfigChanged>();

        // Add systems
        app.add_systems(Startup, load_and_apply_config);
        app.add_systems(
            Update,
            (
                // Change detection systems (emit ConfigChanged)
                detect_window_resize,
                detect_window_move,
                detect_game_config_changes,
                // Reactive settings application
                apply_display_mode
                    .run_if(resource_changed::<super::resources::GameConfig>),
                apply_deferred_mode_change.run_if(
                    |geo: Res<super::resources::SavedWindowedGeometry>| {
                        geo.pending_mode_change.is_some()
                    },
                ),
                // Unified debounce trigger
                mark_save_on_config_changed,
                // Save systems
                save_config_on_debounce_timer
                    .run_if(|timer: Res<super::resources::SaveDebounceTimer>| timer.pending),
                save_config_on_event,
                // Periodic save cache flush (every 2s when dirty)
                periodic_save_flush
                    .run_if(super::save_data::save_cache_is_dirty),
            ),
        );
    }

    fn name(&self) -> &str {
        "ConfigPlugin"
    }
}
