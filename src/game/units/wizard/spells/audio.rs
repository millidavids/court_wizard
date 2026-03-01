use bevy::audio::Volume;
use bevy::prelude::*;

use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;

/// Maximum distance for sound effect attenuation.
/// Effects at this distance or beyond are silent.
const MAX_SFX_DISTANCE: f32 = 10000.0;

/// Preloaded audio handles for spell sound effects.
#[derive(Resource)]
pub(crate) struct SpellSfxAssets {
    pub magic_missile_cast: Handle<AudioSource>,
    pub fireball_cast: Handle<AudioSource>,
    pub fireball_impact: Handle<AudioSource>,
}

/// Loads all spell sound effect assets at startup.
pub(super) fn load_spell_sfx_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SpellSfxAssets {
        magic_missile_cast: asset_server.load("audio/sound_effects/magic_missile_cast.ogg"),
        fireball_cast: asset_server.load("audio/sound_effects/fireball_cast.ogg"),
        fireball_impact: asset_server.load("audio/sound_effects/fireball_impact.ogg"),
    });
}

/// Plays a one-shot sound effect with distance-based volume attenuation from the wizard.
pub(crate) fn play_sfx(
    commands: &mut Commands,
    handle: &Handle<AudioSource>,
    effect_pos: Vec3,
    game_config: &GameConfig,
) {
    let distance = effect_pos.distance(SPELL_ORIGIN);
    let linear = (1.0 - distance / MAX_SFX_DISTANCE).clamp(0.0, 1.0);
    let attenuation = linear * linear * linear * linear; // Squared falloff — distant sounds drop off faster
    let volume = game_config.master_volume * game_config.sfx_volume * attenuation;

    if volume <= 0.0 {
        return;
    }

    commands.spawn((
        AudioPlayer::new(handle.clone()),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(volume)),
        OnGameplayScreen,
    ));
}
