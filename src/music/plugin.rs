use bevy::prelude::*;

use super::resources::{ActiveMusic, MusicFadeIn, MusicFadeOut};
use super::systems;

/// Plugin that manages background music with crossfading between tracks.
///
/// Two music tracks:
/// - Menu music (citadel-of-frozen-ink.ogg): Splash, MainMenu, MetaGame
/// - Gameplay music (fireball_dungeon_mix.ogg): Loading, InGame, Multiplayer
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
                        resource_changed::<State<crate::state::AppState>>.or(
                            |active: Option<Res<ActiveMusic>>| {
                                active.map_or(true, |a| a.current_track.is_none())
                            },
                        ),
                    ),
                    systems::process_music_fade_in
                        .run_if(any_with_component::<MusicFadeIn>),
                    systems::process_music_fade_out
                        .run_if(any_with_component::<MusicFadeOut>),
                    systems::sync_music_volume
                        .run_if(resource_changed::<crate::config::GameConfig>),
                ),
            );
    }
}
