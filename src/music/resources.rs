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

/// Marker component identifying a music entity.
#[derive(Component)]
pub(super) struct MusicEntity;

/// Direction of a music volume fade.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FadeDirection {
    In,
    Out,
}

/// Component for music entities that are fading in or out.
/// Entities fading out are despawned when complete; fade-in components are removed.
#[derive(Component)]
pub(super) struct MusicFade {
    pub(super) timer: Timer,
    pub(super) direction: FadeDirection,
}
