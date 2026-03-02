use bevy::prelude::*;

/// Duration for music fade transitions in seconds.
pub(super) const FADE_DURATION_SECS: f32 = 1.5;

/// Which music track to play.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MusicTrack {
    /// Menu music - plays during Splash, MainMenu, MetaGame.
    Menu,
    /// Gameplay music - plays during Loading, InGame, Multiplayer.
    Gameplay,
}

/// Resource holding loaded music audio handles for both tracks.
#[derive(Resource)]
pub(super) struct MusicAssets {
    pub(super) menu_music: Handle<AudioSource>,
    pub(super) gameplay_music: Handle<AudioSource>,
}

/// Resource tracking which music track is currently active.
#[derive(Resource, Default)]
pub(super) struct ActiveMusic {
    pub(super) current_track: Option<MusicTrack>,
}

/// Marker component identifying a music entity and its track.
#[derive(Component)]
pub(super) struct MusicEntity;

/// Component for music entities that are fading in.
#[derive(Component)]
pub(super) struct MusicFadeIn {
    pub(super) timer: Timer,
}

/// Component for music entities that are fading out (despawned when complete).
#[derive(Component)]
pub(super) struct MusicFadeOut {
    pub(super) timer: Timer,
}
