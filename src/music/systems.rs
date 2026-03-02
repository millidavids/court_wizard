use bevy::audio::Volume;
use bevy::prelude::*;

use crate::config::GameConfig;
use crate::state::AppState;

use super::resources::{
    ActiveMusic, MusicAssets, MusicEntity, MusicFadeIn, MusicFadeOut, MusicTrack, FADE_DURATION_SECS,
};

/// Loads both music track assets at startup.
pub(super) fn load_music_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(MusicAssets {
        menu_music: asset_server.load("audio/citadel-of-frozen-ink.ogg"),
        gameplay_music: asset_server.load("audio/fireball_dungeon_mix.ogg"),
    });
    commands.insert_resource(ActiveMusic::default());
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
    mut active_music: ResMut<ActiveMusic>,
    music_query: Query<Entity, (With<MusicEntity>, Without<MusicFadeOut>)>,
) {
    let Some(music_assets) = music_assets else {
        return;
    };
    let Some(game_config) = game_config else {
        return;
    };

    let desired_track = track_for_state(app_state.get());

    if active_music.current_track == Some(desired_track) {
        return;
    }

    info!(
        "Music transition: {:?} -> {:?}",
        active_music.current_track, desired_track
    );

    // Fade out all existing music entities that aren't already fading out
    for entity in &music_query {
        commands
            .entity(entity)
            .remove::<MusicFadeIn>()
            .insert(MusicFadeOut {
                timer: Timer::from_seconds(FADE_DURATION_SECS, TimerMode::Once),
            });
    }

    // Spawn new music with fade-in (starts at volume 0)
    let handle = match desired_track {
        MusicTrack::Menu => music_assets.menu_music.clone(),
        MusicTrack::Gameplay => music_assets.gameplay_music.clone(),
    };

    let target_volume = game_config.master_volume * game_config.music_volume;

    // If no previous track was playing, skip the fade-in and start at full volume
    let is_first_track = active_music.current_track.is_none();

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
            MusicFadeIn {
                timer: Timer::from_seconds(FADE_DURATION_SECS, TimerMode::Once),
            },
        ));
    }

    active_music.current_track = Some(desired_track);
}

/// Gradually increases volume of fading-in music entities.
pub(super) fn process_music_fade_in(
    mut commands: Commands,
    time: Res<Time>,
    game_config: Option<Res<GameConfig>>,
    mut query: Query<(Entity, &mut MusicFadeIn, &mut AudioSink)>,
) {
    let Some(game_config) = game_config else {
        return;
    };
    let target = game_config.master_volume * game_config.music_volume;

    for (entity, mut fade, mut sink) in &mut query {
        fade.timer.tick(time.delta());
        sink.set_volume(Volume::Linear(target * fade.timer.fraction()));

        if fade.timer.is_finished() {
            commands.entity(entity).remove::<MusicFadeIn>();
        }
    }
}

/// Gradually decreases volume of fading-out music entities, despawning when silent.
pub(super) fn process_music_fade_out(
    mut commands: Commands,
    time: Res<Time>,
    game_config: Option<Res<GameConfig>>,
    mut query: Query<(Entity, &mut MusicFadeOut, &mut AudioSink)>,
) {
    let Some(game_config) = game_config else {
        return;
    };
    let target = game_config.master_volume * game_config.music_volume;

    for (entity, mut fade, mut sink) in &mut query {
        fade.timer.tick(time.delta());
        sink.set_volume(Volume::Linear(target * (1.0 - fade.timer.fraction())));

        if fade.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Syncs volume from GameConfig to active (non-fading) music entities.
pub(super) fn sync_music_volume(
    game_config: Res<GameConfig>,
    mut query: Query<
        &mut AudioSink,
        (
            With<MusicEntity>,
            Without<MusicFadeIn>,
            Without<MusicFadeOut>,
        ),
    >,
) {
    let effective = game_config.master_volume * game_config.music_volume;
    for mut sink in &mut query {
        sink.set_volume(Volume::Linear(effective));
    }
}
