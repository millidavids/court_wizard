use bevy::prelude::*;

/// Resource holding the loaded background music audio handle.
///
/// The audio asset is loaded at startup and stored here for use when entering the game.
#[derive(Resource)]
pub(super) struct BackgroundMusicAssets {
    pub(super) music_handle: Handle<AudioSource>,
}

/// Marker component for the background music entity.
///
/// Used to identify and query the music entity for volume updates.
#[derive(Component)]
pub(super) struct BackgroundMusicEntity;
