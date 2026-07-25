//! Weather effects: lightning, burning patches, SFX.

use bevy::audio::Volume;
use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use super::messages::WeatherChangedMessage;
use super::resources::{WeatherState, WeatherType};
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::pathfinding::StagingAttacker;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_damage_to_unit, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::networking::session::MultiplayerSession;

/// Applies the Drought healing reduction to a heal amount.
/// Returns the (possibly reduced) heal amount.
#[allow(clippy::too_many_arguments)]
pub fn storm_lightning(
    mut commands: Commands,
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut weather: ResMut<WeatherState>,
    game_config: Res<GameConfig>,
    sfx: Res<SpellSfxAssets>,
    mut pending: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (Without<Corpse>, Without<StagingAttacker>),
    >,
    session: Option<Res<MultiplayerSession>>,
) {
    let local_team = local_player_team(session.as_deref());
    // Weather sim is host-only, so the `remote` slot (the opponent's storm) must
    // be attributed to the OTHER player's team — otherwise the host's shield check
    // treats the opponent's storm as friendly and the King-shield is inverted.
    let remote_team = if local_team == Team::Defenders {
        Team::Attackers
    } else {
        Team::Defenders
    };
    let delta = time.delta_secs();

    // Tick each active Storm slot's lightning timer; collect (intensity, owner)
    // for every slot that strikes THIS frame. Both slots can be Storm in a
    // Meteorologist-vs-Meteorologist match, each firing on its own cadence.
    let weather_ref = &mut *weather;
    let mut strikes: Vec<(f32, Team)> = Vec::new();
    for (slot, owner_team) in [
        (&mut weather_ref.local, local_team),
        (&mut weather_ref.remote, remote_team),
    ] {
        if slot.active != Some(WeatherType::Storm) {
            continue;
        }
        slot.lightning_timer -= delta;
        if slot.lightning_timer <= 0.0 {
            // Reset timer (scales with intensity — faster strikes when stronger)
            slot.lightning_timer = THUNDERSTORM_LIGHTNING_INTERVAL / slot.intensity.max(0.01);
            strikes.push((slot.intensity, owner_team));
        }
    }
    if strikes.is_empty() {
        return;
    }

    // Count targets once — strikes deal damage but never despawn units, so the
    // count is stable for the whole loop.
    let target_count = targets.iter().len();
    if target_count == 0 {
        return;
    }

    for (storm_intensity, owner_team) in strikes {
        let rng = &mut game_rng.0;
        let target_index = rng.random_range(0..target_count);

        let Some((entity, transform, target_team, mut health, mut temp_hp, has_shield)) =
            targets.iter_mut().nth(target_index)
        else {
            // A slot's strike found no target this frame; try the next slot.
            continue;
        };

        let strike_pos = transform.translation;

        // Replicate the thunderclap to the opponent (the local sound plays below).
        crate::game::units::wizard::spells::audio::emit_sfx_event(
            &mut pending,
            crate::networking::snapshot::SpellSoundId::WeatherLightningStrike,
            strike_pos,
        );

        // Deal AoE damage at the strike location. apply_spell_damage_with_team
        // blocks only the OPPONENT's storm via the King's shield (owner_team !=
        // target) and records source_team + the spell-damage tally.
        let damage = THUNDERSTORM_LIGHTNING_DAMAGE * storm_intensity;
        apply_spell_damage_with_team(
            &mut commands,
            entity,
            &mut health,
            temp_hp.as_deref_mut(),
            damage,
            crate::game::units::damage::DamageType::Electric,
            has_shield,
            owner_team,
            *target_team,
        );

        // Deal AoE damage to nearby units — also via the team-aware helper so
        // splash victims get the Electric DoT/status stacking and the tally too.
        // The helper does the shield check, so the filter only needs the radius.
        let splash_targets: Vec<Entity> = targets
            .iter()
            .filter(|(e, t, _, _, _, _)| {
                *e != entity && {
                    let dx = strike_pos.x - t.translation.x;
                    let dz = strike_pos.z - t.translation.z;
                    (dx * dx + dz * dz)
                        <= THUNDERSTORM_LIGHTNING_RADIUS * THUNDERSTORM_LIGHTNING_RADIUS
                }
            })
            .map(|(e, _, _, _, _, _)| e)
            .collect();

        let splash_damage = THUNDERSTORM_LIGHTNING_DAMAGE * storm_intensity * 0.5;
        for splash_entity in splash_targets {
            if let Ok((_, _, splash_team, mut h, mut th, splash_shield)) =
                targets.get_mut(splash_entity)
            {
                apply_spell_damage_with_team(
                    &mut commands,
                    splash_entity,
                    &mut h,
                    th.as_deref_mut(),
                    splash_damage,
                    crate::game::units::damage::DamageType::Electric,
                    splash_shield,
                    owner_team,
                    *splash_team,
                );
            }
        }

        // Play lightning rod impact SFX at strike location
        audio::play_impact_sfx_scaled(
            &mut commands,
            &sfx.lightning_rod_impact,
            strike_pos,
            &game_config,
            &sfx,
            0.5,
        );

        // Spawn lightning visual (vertical beam)
        let beam_height = 3000.0;
        let beam_pos = Vec3::new(strike_pos.x, beam_height / 2.0, strike_pos.z);

        let material = materials.add(StandardMaterial {
            base_color: THUNDERSTORM_LIGHTNING_COLOR,
            unlit: true,
            ..default()
        });
        let mesh = meshes.add(Rectangle::new(8.0, beam_height));

        commands.spawn((
            LightningStrike { lifetime: 0.15 },
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(beam_pos),
            OnGameplayScreen,
        ));
    }
}

/// Ticks and despawns burning patches, dealing damage to units inside.
/// Staging attackers (not yet activated at their rally point) are excluded.
pub fn update_burning_patches(
    mut commands: Commands,
    time: Res<Time>,
    mut patches: Query<(Entity, &Transform, &mut BurningPatch)>,
    mut units: Query<
        (
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (Without<Corpse>, Without<StagingAttacker>),
    >,
) {
    let delta = time.delta_secs();

    for (patch_entity, patch_tf, mut patch) in patches.iter_mut() {
        patch.lifetime -= delta;
        if patch.lifetime <= 0.0 {
            commands.entity(patch_entity).try_despawn();
            continue;
        }

        patch.tick_timer -= delta;
        if patch.tick_timer > 0.0 {
            continue;
        }
        patch.tick_timer = BURNING_PATCH_TICK_INTERVAL;

        let patch_pos = patch_tf.translation;
        let radius_sq = patch.radius * patch.radius;

        for (unit_tf, mut health, temp_hp, has_shield) in units.iter_mut() {
            // Burning patches are an unowned environmental weather hazard, so a
            // shielded King is immune (matching pre-King-shield behavior) rather
            // than risk mis-attributing the opponent's patches to the host team.
            if has_shield {
                continue;
            }
            let dx = patch_pos.x - unit_tf.translation.x;
            let dz = patch_pos.z - unit_tf.translation.z;
            if dx * dx + dz * dz <= radius_sq {
                apply_damage_to_unit(
                    &mut health,
                    temp_hp.map(|t| t.into_inner()),
                    patch.damage_per_tick,
                );
            }
        }
    }
}

/// Despawns lightning strike visuals after their lifetime.
pub fn update_lightning_visuals(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut LightningStrike)>,
) {
    let delta = time.delta_secs();
    for (entity, mut strike) in query.iter_mut() {
        strike.lifetime -= delta;
        if strike.lifetime <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Fades burning patch visuals based on remaining lifetime.
pub fn update_burning_patch_visuals(
    mut materials: ResMut<Assets<StandardMaterial>>,
    patches: Query<(&BurningPatch, &MeshMaterial3d<StandardMaterial>)>,
) {
    for (patch, mat_handle) in patches.iter() {
        if let Some(material) = materials.get_mut(&mat_handle.0) {
            let alpha = (patch.lifetime / BURNING_PATCH_LIFETIME).clamp(0.0, 1.0) * 0.4;
            material.base_color = Color::srgba(1.0, 0.4, 0.1, alpha);
        }
    }
}

/// Removes weather status components and despawns hazard entities when LEAVING the
/// Running gameplay state. NOTE: `OnExit(Running)` also fires on pause / spell-book /
/// cauldron (sibling sub-states), so this must NOT touch the persistent weather
/// state or overlays — the status modifiers re-apply on resume from the still-active
/// `WeatherState`. End-of-match VFX teardown lives in `clear_weather_visuals`.
pub fn cleanup_weather(
    mut commands: Commands,
    wet: Query<Entity, With<WetModifier>>,
    cold: Query<Entity, With<ColdModifier>>,
    dry: Query<Entity, With<DryModifier>>,
    charged: Query<Entity, With<ChargedModifier>>,
    patches: Query<Entity, With<BurningPatch>>,
    strikes: Query<Entity, With<LightningStrike>>,
) {
    for entity in wet
        .iter()
        .chain(cold.iter())
        .chain(dry.iter())
        .chain(charged.iter())
    {
        commands.entity(entity).remove::<WetModifier>();
        commands.entity(entity).remove::<ColdModifier>();
        commands.entity(entity).remove::<DryModifier>();
        commands.entity(entity).remove::<ChargedModifier>();
    }
    for entity in patches.iter().chain(strikes.iter()) {
        commands.entity(entity).try_despawn();
    }
}

/// Tears down the weather VFX at the END of a match (`OnEnter(ScoreScreen)`):
/// despawns in-flight particles, resets the sky/ground overlay tints to
/// transparent, and clears the weather state — so nothing lingers over the
/// scoreboard. Deliberately NOT on `OnExit(Running)` (that fires on pause too).
pub fn clear_weather_visuals(
    mut commands: Commands,
    particles: Query<Entity, With<WeatherParticle>>,
    mut overlay_bg: Query<&mut BackgroundColor, With<WeatherOverlay>>,
    ground_overlay: Query<&MeshMaterial3d<StandardMaterial>, With<WeatherGroundOverlay>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut weather: ResMut<WeatherState>,
) {
    for entity in particles.iter() {
        commands.entity(entity).try_despawn();
    }
    if let Ok(mut bg) = overlay_bg.single_mut() {
        *bg = BackgroundColor(Color::BLACK.with_alpha(0.0));
    }
    if let Ok(mat_handle) = ground_overlay.single()
        && let Some(material) = materials.get_mut(&mat_handle.0)
    {
        material.base_color = Color::WHITE.with_alpha(0.0);
    }
    *weather = WeatherState::default();
}

// ---------------------------------------------------------------------------
// Weather ambient SFX
// ---------------------------------------------------------------------------

/// Manages looping weather ambient sound — spawns/despawns on weather change.
pub fn update_weather_sfx(
    mut commands: Commands,
    weather: Res<WeatherState>,
    game_config: Res<GameConfig>,
    sfx: Res<SpellSfxAssets>,
    existing_sfx: Query<(Entity, &WeatherSfx)>,
    mut msg: MessageReader<WeatherChangedMessage>,
) {
    // Only act on weather changes
    if msg.read().next().is_none() {
        return;
    }

    let volume = game_config.effective_sfx_volume() * WEATHER_SFX_VOLUME;

    // Desired set: each unique active weather across both slots that has a looping
    // sound (Drought is silent). Empty when volume is muted.
    let mut desired: Vec<WeatherType> = Vec::new();
    if volume > 0.0 {
        for slot in [&weather.local, &weather.remote] {
            if let Some(w) = slot.active
                && matches!(w, WeatherType::Storm | WeatherType::Blizzard)
                && !desired.contains(&w)
            {
                desired.push(w);
            }
        }
    }

    // Reconcile: keep sounds still wanted (so an unchanged loop never restarts),
    // despawn the rest, and remember what survives.
    let mut still_playing: Vec<WeatherType> = Vec::new();
    for (entity, marker) in existing_sfx.iter() {
        if desired.contains(&marker.0) && !still_playing.contains(&marker.0) {
            still_playing.push(marker.0);
        } else {
            commands.entity(entity).try_despawn();
        }
    }

    // Spawn only the wanted weathers that aren't already playing.
    for w in desired {
        if still_playing.contains(&w) {
            continue;
        }
        let handle = match w {
            WeatherType::Storm => &sfx.rain_persistent,
            WeatherType::Blizzard => &sfx.blizzard_persistent,
            WeatherType::Drought => continue,
        };
        commands.spawn((
            AudioPlayer::new(handle.clone()),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(volume)),
            WeatherSfx(w),
            OnGameplayScreen,
        ));
    }
}
