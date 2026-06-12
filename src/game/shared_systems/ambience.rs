use bevy::audio::Volume;
use bevy::prelude::*;

use super::super::constants::*;

/// Number of units in melee at which the battle sound reaches full intensity.
const BATTLE_AMBIENCE_MAX_UNITS: f32 = 40.0;

/// Overall volume scale for battle ambience (kept subtle so it doesn't overpower spells/music).
const BATTLE_AMBIENCE_VOLUME_SCALE: f32 = 0.15;

/// Maximum distance for battle ambience attenuation (same as spell SFX).
const BATTLE_AMBIENCE_MAX_DISTANCE: f32 = 10000.0;

/// Overall volume scale for the crowd ambience loop.
const CROWD_AMBIENCE_VOLUME_SCALE: f32 = 0.12;

/// Position of the battlefield center for crowd sound attenuation (XZ from staging point).
const BATTLEFIELD_CENTER: Vec3 = Vec3::new(
    STAGING_POINTS[CENTER_STAGING_INDEX].0,
    0.0,
    STAGING_POINTS[CENTER_STAGING_INDEX].1,
);

/// Pre-loaded battle ambience audio.
#[derive(Resource)]
pub struct BattleAmbienceAssets {
    pub battle_audio: Handle<AudioSource>,
    pub crowd_audio: Handle<AudioSource>,
}

/// Marker component for the melee battle ambience audio entity.
#[derive(Component)]
pub(crate) struct BattleAmbienceEntity;

/// Marker component for the crowd ambience audio entity.
#[derive(Component)]
pub(crate) struct CrowdAmbienceEntity;

/// Loads the battle ambience audio assets at startup.
pub fn load_battle_ambience_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BattleAmbienceAssets {
        battle_audio: asset_server.load("audio/sound_effects/battle.ogg"),
        crowd_audio: asset_server.load("audio/sound_effects/angry_crowd.ogg"),
    });
}

/// Scales the looping battle ambience volume based on how many units are in melee combat.
/// Uses distance attenuation from the average melee position to the wizard, so battles
/// closer to the castle sound louder than distant skirmishes.
pub fn update_battle_ambience(
    mut commands: Commands,
    melee_query: Query<(&super::super::units::components::InMelee, &Transform)>,
    mut ambience_query: Query<(Entity, Option<&mut AudioSink>), With<BattleAmbienceEntity>>,
    ambience_assets: Res<BattleAmbienceAssets>,
    game_config: Res<crate::config::GameConfig>,
) {
    let melee_count = melee_query.iter().count() as f32;
    let sfx_volume = game_config.effective_sfx_volume();

    if melee_count > 0.0 && sfx_volume > 0.0 {
        // Compute average position of all melee units for distance attenuation
        let avg_pos = melee_query
            .iter()
            .fold(Vec3::ZERO, |acc, (_, t)| acc + t.translation)
            / melee_count;
        let distance = avg_pos
            .distance(crate::game::units::wizard::spells::utils::local_spell_origin_snapshot());
        let linear = (1.0 - distance / BATTLE_AMBIENCE_MAX_DISTANCE).clamp(0.0, 1.0);
        let attenuation = linear * linear * linear;

        let intensity = (melee_count / BATTLE_AMBIENCE_MAX_UNITS).clamp(0.0, 1.0);
        let volume = sfx_volume * intensity * attenuation * BATTLE_AMBIENCE_VOLUME_SCALE;

        if volume <= 0.0 {
            if let Ok((entity, _)) = ambience_query.single() {
                commands.entity(entity).try_despawn();
            }
            return;
        }

        if let Ok((_entity, sink)) = ambience_query.single_mut() {
            if let Some(mut sink) = sink {
                sink.set_volume(Volume::Linear(volume));
            }
        } else {
            commands.spawn((
                AudioPlayer::new(ambience_assets.battle_audio.clone()),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(volume)),
                BattleAmbienceEntity,
                super::super::components::OnGameplayScreen,
            ));
        }
    } else if let Ok((entity, _)) = ambience_query.single() {
        commands.entity(entity).try_despawn();
    }
}

/// Plays a muffled crowd loop throughout the battle, attenuated from the battlefield center.
/// Spawns on first run, despawns when SFX is muted.
pub fn update_crowd_ambience(
    mut commands: Commands,
    mut crowd_query: Query<(Entity, Option<&mut AudioSink>), With<CrowdAmbienceEntity>>,
    ambience_assets: Res<BattleAmbienceAssets>,
    game_config: Res<crate::config::GameConfig>,
) {
    let sfx_volume = game_config.effective_sfx_volume();

    // Distance from battlefield center to wizard
    let distance = BATTLEFIELD_CENTER
        .distance(crate::game::units::wizard::spells::utils::local_spell_origin_snapshot());
    let linear = (1.0 - distance / BATTLE_AMBIENCE_MAX_DISTANCE).clamp(0.0, 1.0);
    let attenuation = linear * linear * linear;
    let volume = sfx_volume * attenuation * CROWD_AMBIENCE_VOLUME_SCALE;

    if volume > 0.0 {
        if let Ok((_entity, sink)) = crowd_query.single_mut() {
            if let Some(mut sink) = sink {
                sink.set_volume(Volume::Linear(volume));
            }
        } else {
            commands.spawn((
                AudioPlayer::new(ambience_assets.crowd_audio.clone()),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(volume)),
                CrowdAmbienceEntity,
                super::super::components::OnGameplayScreen,
            ));
        }
    } else if let Ok((entity, _)) = crowd_query.single() {
        commands.entity(entity).try_despawn();
    }
}
