use bevy::prelude::*;

use super::resources::BackgroundMusicEntity;
use super::systems;

/// Plugin that manages background music playback.
///
/// Loads the background music at startup and plays it when entering the main menu.
/// The music loops continuously throughout the game and respects volume settings from GameConfig.
///
/// Uses Bevy's GlobalVolume resource for master volume control and individual AudioPlayer
/// volume for music-specific volume, allowing separate SFX volume control in the future.
///
/// Lifecycle:
/// - Startup: Load audio asset
/// - Update: Spawn looping music entity once resources are ready (runs once)
/// - Update: Sync volume from GameConfig to Bevy audio system
pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::load_music_assets)
            .add_systems(
                Update,
                (
                    systems::play_background_music
                        .run_if(not(any_with_component::<BackgroundMusicEntity>)),
                    systems::sync_volume_from_config,
                ),
            );
    }
}
