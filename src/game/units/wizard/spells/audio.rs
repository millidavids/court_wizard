use bevy::audio::Volume;
use bevy::prelude::*;

use crate::config::{GameConfig, WizardType};
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::spell_sync::PendingCastEvents;
use crate::game::units::wizard::spells::utils::local_spell_origin_snapshot;
use crate::networking::snapshot::{CastEventKind, CastEventSnapshot, SpellSoundId};

/// Maximum distance for sound effect attenuation.
/// Effects at this distance or beyond are silent.
const MAX_SFX_DISTANCE: f32 = 10000.0;

/// Audio listener position — the player's local wizard. Read via the shared
/// lock-free snapshot so distance attenuation works for both the single-player
/// listener and the multiplayer guest (whose listener is at the guest wizard).
pub(crate) fn audio_origin() -> Vec3 {
    local_spell_origin_snapshot()
}

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
    pub fart_cast: Handle<AudioSource>,
    pub fart_channeling: Handle<AudioSource>,
    pub cauldron_bubbling: Handle<AudioSource>,
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
    // Weather ambient sounds (Meteorologist)
    pub rain_persistent: Handle<AudioSource>,
    pub blizzard_persistent: Handle<AudioSource>,
    // Gun sound effects (Warglock)
    pub machine_gun_shot: Handle<AudioSource>,
    pub magnum_shot: Handle<AudioSource>,
    pub rocket_launcher_shot: Handle<AudioSource>,
    pub shotgun_shot: Handle<AudioSource>,
    // Boulder impact (thrown rock landing)
    pub boulder_impact: Handle<AudioSource>,
    pub ray_eye_death: Handle<AudioSource>,
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
        fart_cast: asset_server.load("audio/sound_effects/fart_cast.ogg"),
        cauldron_bubbling: asset_server.load("audio/sound_effects/cauldron_bubbling.ogg"),
        fart_channeling: asset_server.load("audio/sound_effects/fart_channeling.ogg"),
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
        wall_of_fire_persistent: asset_server
            .load("audio/sound_effects/wall_of_fire_persistent.ogg"),
        wall_of_stone_cast: asset_server.load("audio/sound_effects/wall_of_stone_cast.ogg"),
        // Weather ambient sounds (Meteorologist)
        rain_persistent: asset_server.load("audio/sound_effects/rain_persistent.ogg"),
        blizzard_persistent: asset_server.load("audio/sound_effects/blizzard_persistent.ogg"),
        // Gun sound effects (Warglock)
        machine_gun_shot: asset_server.load("audio/sound_effects/machine_gun_shot.ogg"),
        magnum_shot: asset_server.load("audio/sound_effects/magnum_shot.ogg"),
        rocket_launcher_shot: asset_server.load("audio/sound_effects/rocket_launcher_shot.ogg"),
        shotgun_shot: asset_server.load("audio/sound_effects/shotgun_shot.ogg"),
        boulder_impact: asset_server.load("audio/sound_effects/boulder_impact.ogg"),
        ray_eye_death: asset_server.load("audio/sound_effects/ray_eye_death.ogg"),
    });
}

/// Resolves the effective audio handle, applying Excremage overrides by sound category.
fn resolve_excremage_handle<'a>(
    handle: &'a Handle<AudioSource>,
    kind: SfxKind,
    config: &GameConfig,
    sfx: &'a SpellSfxAssets,
) -> &'a Handle<AudioSource> {
    if config.wizard_type != WizardType::Excremage {
        return handle;
    }
    match kind {
        SfxKind::Cast => &sfx.fart_cast,
        SfxKind::Impact => &sfx.grease_cast,
        SfxKind::Channel => &sfx.fart_channeling,
    }
}

/// Sound effect category for Excremage override selection.
pub(crate) enum SfxKind {
    /// Spell cast sounds (e.g. fireball launch, chain lightning cast).
    Cast,
    /// Impact/explosion sounds (e.g. fireball impact, squall explosion).
    Impact,
    /// Looping channeled sounds (e.g. disintegrate beam, black hole).
    Channel,
}

/// Plays a one-shot cast sound effect with distance-based volume attenuation from the wizard.
/// Excremage overrides cast sounds with fart_cast.
pub(crate) fn play_sfx(
    commands: &mut Commands,
    handle: &Handle<AudioSource>,
    effect_pos: Vec3,
    game_config: &GameConfig,
    sfx_assets: &SpellSfxAssets,
) {
    let effective = resolve_excremage_handle(handle, SfxKind::Cast, game_config, sfx_assets);
    play_sfx_scaled(commands, effective, effect_pos, game_config, 1.0);
}

/// Plays a one-shot impact/explosion sound effect with distance-based volume attenuation.
/// Excremage overrides impact sounds with grease_cast.
pub(crate) fn play_impact_sfx(
    commands: &mut Commands,
    handle: &Handle<AudioSource>,
    effect_pos: Vec3,
    game_config: &GameConfig,
    sfx_assets: &SpellSfxAssets,
) {
    play_impact_sfx_scaled(commands, handle, effect_pos, game_config, sfx_assets, 1.0);
}

/// Like `play_impact_sfx` but with an additional volume scale factor.
pub(crate) fn play_impact_sfx_scaled(
    commands: &mut Commands,
    handle: &Handle<AudioSource>,
    effect_pos: Vec3,
    game_config: &GameConfig,
    sfx_assets: &SpellSfxAssets,
    volume_scale: f32,
) {
    let effective = resolve_excremage_handle(handle, SfxKind::Impact, game_config, sfx_assets);
    play_sfx_scaled(commands, effective, effect_pos, game_config, volume_scale);
}

/// Plays a one-shot sound effect with distance-based attenuation and an additional volume scale.
pub(crate) fn play_sfx_scaled(
    commands: &mut Commands,
    handle: &Handle<AudioSource>,
    effect_pos: Vec3,
    game_config: &GameConfig,
    volume_scale: f32,
) {
    let distance = effect_pos.distance(audio_origin());
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
/// Excremage overrides all channeling sounds with fart_channeling.
pub(crate) fn play_looping_sfx(
    commands: &mut Commands,
    handle: &Handle<AudioSource>,
    game_config: &GameConfig,
    sfx_assets: &SpellSfxAssets,
) -> Entity {
    let handle = resolve_excremage_handle(handle, SfxKind::Channel, game_config, sfx_assets);
    let volume = game_config.effective_sfx_volume();

    if volume <= 0.0 {
        return commands.spawn((ChannelingSfx, OnGameplayScreen)).id();
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

/// Maps a `SpellSoundId` to the corresponding handle in `SpellSfxAssets`.
/// Used by both the local synced wrappers and `apply_remote_cast_events` on
/// the receiving peer.
pub(crate) fn lookup_sfx_handle<'a>(
    id: SpellSoundId,
    sfx: &'a SpellSfxAssets,
) -> &'a Handle<AudioSource> {
    match id {
        SpellSoundId::MagicMissileCast => &sfx.magic_missile_cast,
        SpellSoundId::FireballCast => &sfx.fireball_cast,
        SpellSoundId::FireballImpact => &sfx.fireball_impact,
        SpellSoundId::ArcaneCrystalCast => &sfx.arcane_crystal_cast,
        SpellSoundId::BanishmentCast => &sfx.banishment_cast,
        SpellSoundId::BattleHymnCast => &sfx.battle_hymn_cast,
        SpellSoundId::BerserkerRageCast => &sfx.berserker_rage_cast,
        SpellSoundId::ChainLightningCast => &sfx.chain_lightning_cast,
        SpellSoundId::HealingPlumeCast => &sfx.healing_plume_cast,
        SpellSoundId::DispelCast => &sfx.dispel_cast,
        SpellSoundId::EntangleCast => &sfx.entangle_cast,
        SpellSoundId::FingerOfDeathCast => &sfx.finger_of_death_cast,
        SpellSoundId::FogCloudCast => &sfx.fog_cloud_cast,
        SpellSoundId::GreaseCast => &sfx.grease_cast,
        SpellSoundId::GuardianCircleCast => &sfx.guardian_circle_cast,
        SpellSoundId::HasteCast => &sfx.haste_cast,
        SpellSoundId::LightningRodImpact => &sfx.lightning_rod_impact,
        SpellSoundId::MarkOfDeathCast => &sfx.mark_of_death_cast,
        SpellSoundId::MindControlCast => &sfx.mind_control_cast,
        SpellSoundId::PlagueWindCast => &sfx.plague_wind_cast,
        SpellSoundId::PolymorphCast => &sfx.polymorph_cast,
        SpellSoundId::RaiseTheDeadCast => &sfx.raise_the_dead_cast,
        SpellSoundId::SleepCast => &sfx.sleep_cast,
        SpellSoundId::SpikeGrowthCast => &sfx.spike_growth_cast,
        SpellSoundId::SquallImpact => &sfx.squall_impact,
        SpellSoundId::TelekinesisCast => &sfx.telekinesis_cast,
        SpellSoundId::TeleportCast => &sfx.teleport_cast,
        SpellSoundId::WallOfStoneCast => &sfx.wall_of_stone_cast,
        SpellSoundId::BoulderImpact => &sfx.boulder_impact,
        SpellSoundId::RayEyeDeath => &sfx.ray_eye_death,
        SpellSoundId::DisintegrateChannel => &sfx.disintegrate_channel,
        SpellSoundId::MachineGunShot => &sfx.machine_gun_shot,
        SpellSoundId::MagnumShot => &sfx.magnum_shot,
        SpellSoundId::ShotgunShot => &sfx.shotgun_shot,
        SpellSoundId::RocketShot => &sfx.rocket_launcher_shot,
        // Flamethrower uses the same looping channel sound locally; reuse it for
        // the cross-peer burst cue.
        SpellSoundId::FlamethrowerBurst => &sfx.disintegrate_channel,
        SpellSoundId::WeatherLightningStrike => &sfx.lightning_rod_impact,
    }
}

/// Returns the `SfxKind` used for Excremage handle substitution given a
/// `SpellSoundId`. Names ending in `Impact` map to `Impact`; everything else
/// is treated as a `Cast` (we have no looping sounds in this enum).
fn sound_id_kind(id: SpellSoundId) -> SfxKind {
    match id {
        SpellSoundId::FireballImpact
        | SpellSoundId::LightningRodImpact
        | SpellSoundId::SquallImpact
        | SpellSoundId::BoulderImpact => SfxKind::Impact,
        SpellSoundId::DisintegrateChannel => SfxKind::Channel,
        _ => SfxKind::Cast,
    }
}

/// Plays a one-shot SFX locally **and** emits a `SfxOneShot` cast event so
/// the remote peer plays the same sound. Use this in casting handlers in
/// place of `play_sfx` whenever the sound should be heard cross-client.
///
/// Falls back to local-only when `PendingCastEvents` is in single-player
/// mode (so single-player behaviour is unchanged).
pub(crate) fn play_sfx_synced(
    commands: &mut Commands,
    pending: &mut PendingCastEvents,
    sound_id: SpellSoundId,
    effect_pos: Vec3,
    game_config: &GameConfig,
    sfx_assets: &SpellSfxAssets,
) {
    play_sfx_synced_scaled(
        commands,
        pending,
        sound_id,
        effect_pos,
        game_config,
        sfx_assets,
        1.0,
    );
}

/// Like `play_sfx_synced` but with a volume scale factor.
#[allow(clippy::too_many_arguments)]
pub(crate) fn play_sfx_synced_scaled(
    commands: &mut Commands,
    pending: &mut PendingCastEvents,
    sound_id: SpellSoundId,
    effect_pos: Vec3,
    game_config: &GameConfig,
    sfx_assets: &SpellSfxAssets,
    volume_scale: f32,
) {
    let handle = lookup_sfx_handle(sound_id, sfx_assets);
    let kind = sound_id_kind(sound_id);
    let effective = resolve_excremage_handle(handle, kind, game_config, sfx_assets);
    play_sfx_scaled(commands, effective, effect_pos, game_config, volume_scale);

    if pending.mp_active {
        pending.events.push(CastEventSnapshot {
            kind: CastEventKind::SfxOneShot as u8,
            subkind: sound_id as u8,
            x: effect_pos.x,
            y: effect_pos.y,
            z: effect_pos.z,
            extra: [volume_scale, 0.0, 0.0, 0.0],
        });
    }
}

/// Emits a `SfxOneShot` cast event so the remote peer plays `sound_id`, WITHOUT
/// playing it locally. Use when the local sound is handled separately (e.g. a
/// looping channel sound played via `play_looping_sfx`) but the opponent should
/// still hear it fire.
pub(crate) fn emit_sfx_event(
    pending: &mut PendingCastEvents,
    sound_id: SpellSoundId,
    effect_pos: Vec3,
) {
    if pending.mp_active {
        pending.events.push(CastEventSnapshot {
            kind: CastEventKind::SfxOneShot as u8,
            subkind: sound_id as u8,
            x: effect_pos.x,
            y: effect_pos.y,
            z: effect_pos.z,
            extra: [1.0, 0.0, 0.0, 0.0],
        });
    }
}

/// Receiver-side helper: plays a `SfxOneShot` event arriving from the remote
/// peer. Looks up the local handle and routes through `play_sfx_scaled` so
/// the same distance attenuation runs against the local listener (i.e. the
/// receiving peer's own wizard). Excremage substitution applies *locally*
/// because each peer renders sound through their own configured wizard.
pub(crate) fn play_remote_sfx(
    commands: &mut Commands,
    sound_id: SpellSoundId,
    effect_pos: Vec3,
    volume_scale: f32,
    game_config: &GameConfig,
    sfx_assets: &SpellSfxAssets,
    caster_is_excremage: bool,
) {
    let handle = lookup_sfx_handle(sound_id, sfx_assets);
    let kind = sound_id_kind(sound_id);
    // Substitute the Excremage fart/grease sounds when the REMOTE caster is an
    // Excremage (so the opponent's spells sound right on this screen too), as
    // well as via the normal local-config path.
    let effective = if caster_is_excremage {
        match kind {
            SfxKind::Cast => &sfx_assets.fart_cast,
            SfxKind::Impact => &sfx_assets.grease_cast,
            SfxKind::Channel => &sfx_assets.fart_channeling,
        }
    } else {
        resolve_excremage_handle(handle, kind, game_config, sfx_assets)
    };
    play_sfx_scaled(commands, effective, effect_pos, game_config, volume_scale);
}

/// Spawns a looping sound effect with distance-based volume attenuation.
/// Excremage overrides all channeling sounds with fart_channeling.
pub(crate) fn play_looping_sfx_at(
    commands: &mut Commands,
    handle: &Handle<AudioSource>,
    effect_pos: Vec3,
    game_config: &GameConfig,
    sfx_assets: &SpellSfxAssets,
) -> Entity {
    let handle = resolve_excremage_handle(handle, SfxKind::Channel, game_config, sfx_assets);
    let distance = effect_pos.distance(audio_origin());
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
