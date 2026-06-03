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
use crate::game::units::components::{Corpse, Health, TemporaryHitPoints, apply_damage_to_unit};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};

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
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        Without<Corpse>,
    >,
) {
    if weather.active != Some(WeatherType::Storm) {
        return;
    }

    weather.lightning_timer -= time.delta_secs();
    if weather.lightning_timer > 0.0 {
        return;
    }

    // Reset timer (scales with intensity — faster strikes at higher intensity)
    weather.lightning_timer = THUNDERSTORM_LIGHTNING_INTERVAL / weather.intensity;

    // Pick a random target
    let target_count = targets.iter().len();
    if target_count == 0 {
        return;
    }
    let rng = &mut game_rng.0;
    let target_index = rng.random_range(0..target_count);

    let Some((entity, transform, mut health, temp_hp, has_shield)) =
        targets.iter_mut().nth(target_index)
    else {
        return;
    };

    let strike_pos = transform.translation;

    // Replicate the thunderclap to the opponent (the local sound plays below).
    crate::game::units::wizard::spells::audio::emit_sfx_event(
        &mut pending,
        crate::networking::snapshot::SpellSoundId::WeatherLightningStrike,
        strike_pos,
    );

    // Deal AoE damage at strike location
    if !has_shield {
        let damage = THUNDERSTORM_LIGHTNING_DAMAGE * weather.intensity;
        apply_damage_to_unit(&mut health, temp_hp.map(|t| t.into_inner()), damage);
        commands
            .entity(entity)
            .insert(crate::game::units::components::PendingDamageEffect {
                damage_type: crate::game::units::damage::DamageType::Electric,
                damage,
            });
    }

    // Deal AoE damage to nearby units
    let splash_targets: Vec<Entity> = targets
        .iter()
        .filter(|(e, t, _, _, shield)| {
            *e != entity && !shield && {
                let dx = strike_pos.x - t.translation.x;
                let dz = strike_pos.z - t.translation.z;
                (dx * dx + dz * dz) <= THUNDERSTORM_LIGHTNING_RADIUS * THUNDERSTORM_LIGHTNING_RADIUS
            }
        })
        .map(|(e, _, _, _, _)| e)
        .collect();

    let splash_damage = THUNDERSTORM_LIGHTNING_DAMAGE * weather.intensity * 0.5;
    for splash_entity in splash_targets {
        if let Ok((_, _, mut h, th, _)) = targets.get_mut(splash_entity) {
            apply_damage_to_unit(&mut h, th.map(|t| t.into_inner()), splash_damage);
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

/// Ticks and despawns burning patches, dealing damage to units inside.
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
        Without<Corpse>,
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

/// Cleans up all weather status components and patches when exiting gameplay.
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

// ---------------------------------------------------------------------------
// Weather ambient SFX
// ---------------------------------------------------------------------------

/// Manages looping weather ambient sound — spawns/despawns on weather change.
pub fn update_weather_sfx(
    mut commands: Commands,
    weather: Res<WeatherState>,
    game_config: Res<GameConfig>,
    sfx: Res<SpellSfxAssets>,
    existing_sfx: Query<Entity, With<WeatherSfx>>,
    mut msg: MessageReader<WeatherChangedMessage>,
) {
    // Only act on weather changes
    if msg.read().next().is_none() {
        return;
    }

    // Despawn existing weather sound
    for entity in existing_sfx.iter() {
        commands.entity(entity).try_despawn();
    }

    // Spawn new looping sound if weather is active
    let handle = match weather.active {
        Some(WeatherType::Storm) => Some(&sfx.rain_persistent),
        Some(WeatherType::Blizzard) => Some(&sfx.blizzard_persistent),
        _ => None,
    };

    if let Some(handle) = handle {
        let volume = game_config.effective_sfx_volume() * WEATHER_SFX_VOLUME;
        if volume > 0.0 {
            commands.spawn((
                AudioPlayer::new(handle.clone()),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(volume)),
                WeatherSfx,
                OnGameplayScreen,
            ));
        }
    }
}
