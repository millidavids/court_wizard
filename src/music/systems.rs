use bevy::audio::{GlobalVolume, Volume};
use bevy::prelude::*;

use crate::config::GameConfig;

use super::resources::{BackgroundMusicAssets, BackgroundMusicEntity};

/// Loads the background music audio asset at startup.
///
/// The audio handle is stored in the BackgroundMusicAssets resource for later use.
pub(super) fn load_music_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let music_handle = asset_server.load("fireball_dungeon_mix.ogg");
    commands.insert_resource(BackgroundMusicAssets { music_handle });
    info!("Music asset loading: fireball_dungeon_mix.ogg");
}

/// Spawns the background music entity once all resources are ready.
///
/// The music is configured to loop continuously. Volume is controlled by:
/// - Bevy's GlobalVolume resource (master_volume from GameConfig)
/// - Individual AudioPlayer volume (music_volume from GameConfig)
///
/// Runs in Update with a run_if condition to only execute when music doesn't exist yet.
pub(super) fn play_background_music(
    mut commands: Commands,
    music_assets: Option<Res<BackgroundMusicAssets>>,
    game_config: Option<Res<GameConfig>>,
) {
    // Wait for music assets to be loaded
    let Some(music_assets) = music_assets else {
        return;
    };

    // Wait for game config to be loaded
    let Some(game_config) = game_config else {
        return;
    };

    // Spawn looping music entity (no cleanup marker - persists across states)
    // GlobalVolume handles master_volume, individual volume handles music_volume
    commands.spawn((
        BackgroundMusicEntity,
        AudioPlayer::new(music_assets.music_handle.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(game_config.music_volume)),
    ));

    info!(
        "Background music spawned: master_volume={:.2}, music_volume={:.2}",
        game_config.master_volume, game_config.music_volume
    );
}

/// Syncs volume from GameConfig to Bevy's audio system.
///
/// Monitors the GameConfig resource and updates audio volumes whenever settings change:
/// - GlobalVolume.volume = master_volume (affects all audio)
/// - AudioPlayer volume = music_volume (affects only music)
///
/// Also handles initial volume setup when GameConfig first becomes available.
/// This allows the settings UI to control volumes in real-time.
pub(super) fn sync_volume_from_config(
    game_config: Option<Res<GameConfig>>,
    mut global_volume: ResMut<GlobalVolume>,
    mut music_query: Query<&mut AudioSink, With<BackgroundMusicEntity>>,
    mut last_synced: Local<Option<(f32, f32)>>,
) {
    // Wait for game config to be loaded
    let Some(game_config) = game_config else {
        return;
    };

    let current_volumes = (game_config.master_volume, game_config.music_volume);

    // Check if volumes actually changed since last sync
    // This prevents unnecessary updates during state transitions
    let volumes_changed = match *last_synced {
        Some((last_master, last_music)) => {
            (last_master - current_volumes.0).abs() > 0.001
                || (last_music - current_volumes.1).abs() > 0.001
        }
        None => true, // First run, always sync
    };

    if !volumes_changed {
        return;
    }

    // Update global master volume
    global_volume.volume = Volume::Linear(current_volumes.0);

    // Update music-specific volume
    for mut sink in &mut music_query {
        sink.set_volume(Volume::Linear(current_volumes.1));
    }

    // Cache the values we just applied
    *last_synced = Some(current_volumes);

    info!(
        "Audio volumes synced: master={:.2}, music={:.2}",
        current_volumes.0, current_volumes.1
    );
}
