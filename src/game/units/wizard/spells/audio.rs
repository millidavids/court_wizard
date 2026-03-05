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
    pub arcane_crystal_cast: Handle<AudioSource>,
    pub banishment_cast: Handle<AudioSource>,
    pub battle_hymn_cast: Handle<AudioSource>,
    pub berserker_rage_cast: Handle<AudioSource>,
    pub black_hole_persistent: Handle<AudioSource>,
    pub chain_lightning_cast: Handle<AudioSource>,
    pub healing_plume_cast: Handle<AudioSource>,
    pub disintegrate_channel: Handle<AudioSource>,
    pub dispel_cast: Handle<AudioSource>,
    pub entangle_cast: Handle<AudioSource>,
    pub finger_of_death_cast: Handle<AudioSource>,
    pub fog_cloud_cast: Handle<AudioSource>,
    pub grease_cast: Handle<AudioSource>,
    pub guardian_circle_cast: Handle<AudioSource>,
    pub haste_cast: Handle<AudioSource>,
    pub lightning_rod_impact: Handle<AudioSource>,
    pub mark_of_death_cast: Handle<AudioSource>,
    pub mind_control_cast: Handle<AudioSource>,
    pub plague_wind_cast: Handle<AudioSource>,
    pub polymorph_cast: Handle<AudioSource>,
    pub raise_the_dead_cast: Handle<AudioSource>,
    pub sleep_cast: Handle<AudioSource>,
    pub spike_growth_cast: Handle<AudioSource>,
    pub squall_impact: Handle<AudioSource>,
    pub telekinesis_cast: Handle<AudioSource>,
    pub teleport_cast: Handle<AudioSource>,
    pub wall_of_fire_persistent: Handle<AudioSource>,
    pub wall_of_stone_cast: Handle<AudioSource>,
}

/// Loads all spell sound effect assets at startup.
pub(super) fn load_spell_sfx_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SpellSfxAssets {
        magic_missile_cast: asset_server.load("audio/sound_effects/magic_missile_cast.ogg"),
        fireball_cast: asset_server.load("audio/sound_effects/fireball_cast.ogg"),
        fireball_impact: asset_server.load("audio/sound_effects/fireball_impact.ogg"),
        arcane_crystal_cast: asset_server.load("audio/sound_effects/arcane_crystal_cast.ogg"),
        banishment_cast: asset_server.load("audio/sound_effects/banishment_cast.ogg"),
        battle_hymn_cast: asset_server.load("audio/sound_effects/battle_hymn_cast.ogg"),
        berserker_rage_cast: asset_server.load("audio/sound_effects/berserker_rage_cast.ogg"),
        black_hole_persistent: asset_server.load("audio/sound_effects/black_hole_persistent.ogg"),
        chain_lightning_cast: asset_server.load("audio/sound_effects/chain_lightning_cast.ogg"),
        healing_plume_cast: asset_server.load("audio/sound_effects/healing_plume_cast.ogg"),
        disintegrate_channel: asset_server.load("audio/sound_effects/disintegrate_channel.ogg"),
        dispel_cast: asset_server.load("audio/sound_effects/dispel_cast.ogg"),
        entangle_cast: asset_server.load("audio/sound_effects/entangle_cast.ogg"),
        finger_of_death_cast: asset_server.load("audio/sound_effects/finger_of_death_cast.ogg"),
        fog_cloud_cast: asset_server.load("audio/sound_effects/fog_cloud_cast.ogg"),
        grease_cast: asset_server.load("audio/sound_effects/grease_cast.ogg"),
        guardian_circle_cast: asset_server.load("audio/sound_effects/guardian_circle_cast.ogg"),
        haste_cast: asset_server.load("audio/sound_effects/haste_cast.ogg"),
        lightning_rod_impact: asset_server.load("audio/sound_effects/lightning_rod_impact.ogg"),
        mark_of_death_cast: asset_server.load("audio/sound_effects/mark_of_death_cast.ogg"),
        mind_control_cast: asset_server.load("audio/sound_effects/mind_control_cast.ogg"),
        plague_wind_cast: asset_server.load("audio/sound_effects/plague_wind_cast.ogg"),
        polymorph_cast: asset_server.load("audio/sound_effects/polymorph_cast.ogg"),
        raise_the_dead_cast: asset_server.load("audio/sound_effects/raise_the_dead_cast.ogg"),
        sleep_cast: asset_server.load("audio/sound_effects/sleep_cast.ogg"),
        spike_growth_cast: asset_server.load("audio/sound_effects/spike_growth_cast.ogg"),
        squall_impact: asset_server.load("audio/sound_effects/squall_impact.ogg"),
        telekinesis_cast: asset_server.load("audio/sound_effects/telekinesis_cast.ogg"),
        teleport_cast: asset_server.load("audio/sound_effects/teleport_cast.ogg"),
        wall_of_fire_persistent: asset_server.load("audio/sound_effects/wall_of_fire_persistent.ogg"),
        wall_of_stone_cast: asset_server.load("audio/sound_effects/wall_of_stone_cast.ogg"),
    });
}

/// Plays a one-shot sound effect with distance-based volume attenuation from the wizard.
pub(crate) fn play_sfx(
    commands: &mut Commands,
    handle: &Handle<AudioSource>,
    effect_pos: Vec3,
    game_config: &GameConfig,
) {
    play_sfx_scaled(commands, handle, effect_pos, game_config, 1.0);
}

/// Plays a one-shot sound effect with distance-based attenuation and an additional volume scale.
pub(crate) fn play_sfx_scaled(
    commands: &mut Commands,
    handle: &Handle<AudioSource>,
    effect_pos: Vec3,
    game_config: &GameConfig,
    volume_scale: f32,
) {
    let distance = effect_pos.distance(SPELL_ORIGIN);
    let linear = (1.0 - distance / MAX_SFX_DISTANCE).clamp(0.0, 1.0);
    let attenuation = linear * linear * linear * linear * linear * linear; // steep falloff for distant sounds
    let volume = game_config.effective_sfx_volume() * attenuation * volume_scale;

    if volume <= 0.0 {
        return;
    }

    commands.spawn((
        AudioPlayer::new(handle.clone()),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(volume)),
        OnGameplayScreen,
    ));
}

/// Marker component for a looping channeling sound effect entity.
#[derive(Component)]
pub(crate) struct ChannelingSfx;

/// Spawns a looping sound effect that plays until the entity is despawned.
pub(crate) fn play_looping_sfx(
    commands: &mut Commands,
    handle: &Handle<AudioSource>,
    game_config: &GameConfig,
) -> Entity {
    let volume = game_config.effective_sfx_volume();

    if volume <= 0.0 {
        return commands
            .spawn((ChannelingSfx, OnGameplayScreen))
            .id();
    }

    commands
        .spawn((
            AudioPlayer::new(handle.clone()),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(volume)),
            ChannelingSfx,
            OnGameplayScreen,
        ))
        .id()
}

/// Spawns a looping sound effect with distance-based volume attenuation.
pub(crate) fn play_looping_sfx_at(
    commands: &mut Commands,
    handle: &Handle<AudioSource>,
    effect_pos: Vec3,
    game_config: &GameConfig,
) -> Entity {
    let distance = effect_pos.distance(SPELL_ORIGIN);
    let linear = (1.0 - distance / MAX_SFX_DISTANCE).clamp(0.0, 1.0);
    let attenuation = linear * linear * linear * linear * linear * linear;
    let volume = game_config.effective_sfx_volume() * attenuation;

    if volume <= 0.0 {
        return commands.spawn(OnGameplayScreen).id();
    }

    commands
        .spawn((
            AudioPlayer::new(handle.clone()),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(volume)),
            OnGameplayScreen,
        ))
        .id()
}
