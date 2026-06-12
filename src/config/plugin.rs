use bevy::prelude::*;

use super::systems::*;

/// Configuration plugin for managing game settings on disk.
///
/// This plugin provides a complete configuration system that:
/// - Loads configuration from disk at startup
/// - Applies settings to Bevy components (Window, GameConfig, etc.)
/// - **Bevy components are the single source of truth** (no duplicate state)
/// - Implements unified debouncing for all config changes
/// - Persists changes to disk after 2s of inactivity
/// - Flushes all pending saves on app exit (Alt+F4, window close, etc.)
#[derive(Default)]
pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        // Insert resources
        app.init_resource::<super::resources::SaveDebounceTimer>();
        app.init_resource::<super::resources::ActiveSave>();
        app.init_resource::<super::input_bindings::InputBindings>();
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
                detect_input_bindings_changes,
                // Reactive settings application
                apply_display_mode.run_if(resource_changed::<super::resources::GameConfig>),
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
                periodic_save_flush.run_if(super::save_data::save_cache_is_dirty),
            ),
        );

        // Flush saves on app exit (catches Alt+F4, window close, etc.), then
        // immediately force the OS process to terminate — Bevy's own shutdown
        // path can stall for tens of seconds on macOS after a gameplay
        // session, never returning from `app.run()`.
        app.add_systems(Last, (save_on_exit, force_exit_after_save).chain());
    }

    fn name(&self) -> &str {
        "ConfigPlugin"
    }
}
