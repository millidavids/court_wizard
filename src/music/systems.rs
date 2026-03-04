use bevy::audio::Volume;
use bevy::prelude::*;

use crate::config::GameConfig;
use crate::state::AppState;

use super::resources::{
    FADE_DURATION_SECS, FadeDirection, MusicAssets, MusicEntity, MusicFade, MusicTrack,
};

/// Loads both music track assets at startup.
pub(super) fn load_music_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(MusicAssets {
        menu_music: asset_server.load("audio/music/citadel-of-frozen-ink.ogg"),
        gameplay_music: asset_server.load("audio/music/fireball_dungeon_mix.ogg"),
    });
    info!("Music assets loading: menu and gameplay tracks");
}

/// Maps an AppState to the appropriate music track.
fn track_for_state(state: &AppState) -> MusicTrack {
    match state {
        AppState::Splash | AppState::MainMenu | AppState::MetaGame => MusicTrack::Menu,
        AppState::Loading
        | AppState::InGame
        | AppState::MultiplayerLoading
        | AppState::MultiplayerGame => MusicTrack::Gameplay,
    }
}

/// Detects music zone changes and triggers crossfade transitions.
///
/// When the AppState maps to a different track than what's currently playing,
/// fades out existing music and spawns the new track with a fade-in.
pub(super) fn check_music_transition(
    mut commands: Commands,
    app_state: Res<State<AppState>>,
    music_assets: Option<Res<MusicAssets>>,
    game_config: Option<Res<GameConfig>>,
    mut previous_track: Local<Option<MusicTrack>>,
    music_query: Query<Entity, (With<MusicEntity>, Without<MusicFade>)>,
) {
    let Some(music_assets) = music_assets else {
        return;
    };
    let Some(game_config) = game_config else {
        return;
    };

    let desired_track = track_for_state(app_state.get());

    if *previous_track == Some(desired_track) {
        return;
    }

    info!(
        "Music transition: {:?} -> {:?}",
        *previous_track, desired_track
    );

    // Fade out all existing music entities that aren't already fading
    for entity in &music_query {
        commands.entity(entity).insert(MusicFade {
            timer: Timer::from_seconds(FADE_DURATION_SECS, TimerMode::Once),
            direction: FadeDirection::Out,
        });
    }

    // Spawn new music with fade-in (starts at volume 0)
    let handle = match desired_track {
        MusicTrack::Menu => music_assets.menu_music.clone(),
        MusicTrack::Gameplay => music_assets.gameplay_music.clone(),
    };

    let target_volume = game_config.effective_music_volume();

    // If no previous track was playing, skip the fade-in and start at full volume
    let is_first_track = previous_track.is_none();

    if is_first_track {
        commands.spawn((
            MusicEntity,
            AudioPlayer::new(handle),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(target_volume)),
        ));
    } else {
        commands.spawn((
            MusicEntity,
            AudioPlayer::new(handle),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(0.0)),
            MusicFade {
                timer: Timer::from_seconds(FADE_DURATION_SECS, TimerMode::Once),
                direction: FadeDirection::In,
            },
        ));
    }

    *previous_track = Some(desired_track);
}

/// Processes music fade-in and fade-out transitions.
/// Fade-in: gradually increases volume, removes MusicFade when complete.
/// Fade-out: gradually decreases volume, despawns entity when complete.
pub(super) fn process_music_fade(
    mut commands: Commands,
    time: Res<Time>,
    game_config: Option<Res<GameConfig>>,
    mut query: Query<(Entity, &mut MusicFade, &mut AudioSink)>,
) {
    let Some(game_config) = game_config else {
        return;
    };
    let target = game_config.effective_music_volume();

    for (entity, mut fade, mut sink) in &mut query {
        fade.timer.tick(time.delta());

        let fraction = match fade.direction {
            FadeDirection::In => fade.timer.fraction(),
            FadeDirection::Out => 1.0 - fade.timer.fraction(),
        };
        sink.set_volume(Volume::Linear(target * fraction));

        if fade.timer.is_finished() {
            match fade.direction {
                FadeDirection::In => {
                    commands.entity(entity).remove::<MusicFade>();
                }
                FadeDirection::Out => {
                    commands.entity(entity).try_despawn();
                }
            }
        }
    }
}

/// Syncs volume from GameConfig to active (non-fading) music entities.
pub(super) fn sync_music_volume(
    game_config: Res<GameConfig>,
    mut query: Query<&mut AudioSink, (With<MusicEntity>, Without<MusicFade>)>,
) {
    let effective = game_config.effective_music_volume();
    for mut sink in &mut query {
        sink.set_volume(Volume::Linear(effective));
    }
}
