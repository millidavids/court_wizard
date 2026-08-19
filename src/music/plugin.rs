use bevy::prelude::*;

use super::resources::MusicFade;
use super::systems;

/// Plugin that manages background music with crossfading between tracks.
///
/// Two music tracks:
/// - Menu music (pixel_river_loopable.ogg): Splash, MainMenu, MetaGame
/// - Gameplay music (sunshine_skirmish_loopable.ogg): Loading, InGame, Multiplayer
///
/// Transitions between tracks use smooth fade-in/fade-out crossfades.
pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::load_music_assets)
            .add_systems(
                Update,
                (
                    systems::check_music_transition.run_if(
                        resource_changed::<State<crate::state::AppState>>
                            .or_else(resource_added::<super::resources::MusicAssets>),
                    ),
                    systems::process_music_fade.run_if(any_with_component::<MusicFade>),
                    systems::sync_music_volume
                        .run_if(resource_changed::<crate::config::GameConfig>),
                ),
            );
    }
}
