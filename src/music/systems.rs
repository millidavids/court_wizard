use bevy::audio::Volume;
use bevy::prelude::*;

use crate::config::GameConfig;

use super::resources::{BackgroundMusicAssets, BackgroundMusicEntity};

/// Loads the background music audio asset at startup.
///
/// The audio handle is stored in the BackgroundMusicAssets resource for later use.
pub(super) fn load_music_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let music_handle = asset_server.load("audio/fireball_dungeon_mix.ogg");
    commands.insert_resource(BackgroundMusicAssets { music_handle });
    info!("Music asset loading: audio/fireball_dungeon_mix.ogg");
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
        PlaybackSettings::LOOP.with_volume(Volume::Linear(game_config.master_volume * game_config.music_volume)),
    ));

    info!(
        "Background music spawned: master_volume={:.2}, music_volume={:.2}",
        game_config.master_volume, game_config.music_volume
    );
}

/// Syncs volume from GameConfig to Bevy's audio system.
///
/// Gated by `resource_changed::<GameConfig>` run condition so this only
/// executes when GameConfig actually changes, not every frame.
///
/// Each channel's effective volume is its own slider multiplied by the master volume:
/// - Music effective volume = master_volume * music_volume
/// - SFX effective volume = master_volume * sfx_volume (when SFX is implemented)
pub(super) fn sync_volume_from_config(
    game_config: Res<GameConfig>,
    mut music_query: Query<&mut AudioSink, With<BackgroundMusicEntity>>,
) {
    let effective_music = game_config.master_volume * game_config.music_volume;
    for mut sink in &mut music_query {
        sink.set_volume(Volume::Linear(effective_music));
    }

    debug!(
        "Audio volumes synced: master={:.2}, music={:.2} (effective={:.2})",
        game_config.master_volume, game_config.music_volume, effective_music
    );
}
