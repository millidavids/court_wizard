use bevy::audio::Volume;
use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use super::messages::WeatherChangedMessage;
use super::resources::{WeatherState, WeatherType};
use crate::config::GameConfig;
use crate::config::input_bindings::InputBindings;
use crate::game::components::OnGameplayScreen;
use crate::game::units::components::{
    Corpse, ElectricCharge, Health, TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{Mana, Wizard};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};

/// Applies the Drought healing reduction to a heal amount.
/// Returns the (possibly reduced) heal amount.
pub(crate) fn apply_dry_healing_reduction(amount: f32, is_dry: bool) -> f32 {
    if is_dry {
        amount * (1.0 - DRY_HEALING_REDUCTION)
    } else {
        amount
    }
}

/// Shared weather-switching logic used by both keyboard and button handlers.
/// Returns true if the switch was performed, false if blocked (cooldown, mana, etc.).
pub(crate) fn try_switch_weather(
    weather: &mut WeatherState,
    mana: &mut Mana,
    requested: WeatherType,
    writer: &mut MessageWriter<WeatherChangedMessage>,
) -> bool {
    if weather.cooldown > 0.0 {
        return false;
    }

    // Pressing the active weather clears it
    if weather.active == Some(requested) {
        weather.active = None;
        weather.intensity = INTENSITY_MIN;
        weather.time_active = 0.0;
        weather.cooldown = WEATHER_SWITCH_COOLDOWN;
        writer.write(WeatherChangedMessage);
        return true;
    }

    // Check mana
    if mana.current < WEATHER_MANA_COST {
        return false;
    }
    mana.consume(WEATHER_MANA_COST);

    weather.active = Some(requested);
    weather.intensity = INTENSITY_MIN;
    weather.time_active = 0.0;
    weather.cooldown = WEATHER_SWITCH_COOLDOWN;
    weather.lightning_timer = THUNDERSTORM_LIGHTNING_INTERVAL;
    writer.write(WeatherChangedMessage);
    true
}

/// Resets weather state when entering gameplay.
pub fn reset_weather_state(mut weather: ResMut<WeatherState>) {
    *weather = WeatherState::default();
}

/// Handles weather key input to change weather.
pub fn handle_weather_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut weather: ResMut<WeatherState>,
    mut mana_query: Query<&mut Mana, With<Wizard>>,
    mut writer: MessageWriter<WeatherChangedMessage>,
) {
    let weather_keys: [(Option<KeyCode>, WeatherType); 3] = [
        (bindings.meteorologist.weather_1, WeatherType::Storm),
        (bindings.meteorologist.weather_2, WeatherType::Blizzard),
        (bindings.meteorologist.weather_3, WeatherType::Drought),
    ];

    let mut requested = None;
    for (key_opt, weather_type) in weather_keys {
        if let Some(key) = key_opt {
            if keyboard.just_pressed(key) {
                requested = Some(weather_type);
                break;
            }
        }
    }

    let Some(requested) = requested else { return };

    let Ok(mut mana) = mana_query.single_mut() else {
        return;
    };
    try_switch_weather(&mut weather, &mut mana, requested, &mut writer);
}

/// Ticks cooldown and intensity timers.
pub fn tick_weather_timers(time: Res<Time>, mut weather: ResMut<WeatherState>) {
    let delta = time.delta_secs();

    // Tick cooldown
    if weather.cooldown > 0.0 {
        weather.cooldown = (weather.cooldown - delta).max(0.0);
    }

    // Tick intensity ramp
    if weather.active.is_some() {
        weather.time_active += delta;
        let t = (weather.time_active / INTENSITY_RAMP_TIME).min(1.0);
        weather.intensity = INTENSITY_MIN + (INTENSITY_MAX - INTENSITY_MIN) * t;
    }
}

/// Applies or removes weather status components on all living units.
#[allow(clippy::too_many_arguments)]
pub fn apply_weather_status(
    mut commands: Commands,
    weather: Res<WeatherState>,
    units_without_wet: Query<Entity, (Without<Corpse>, Without<WetModifier>, With<Health>)>,
    units_without_cold: Query<Entity, (Without<Corpse>, Without<ColdModifier>, With<Health>)>,
    units_without_dry: Query<Entity, (Without<Corpse>, Without<DryModifier>, With<Health>)>,
    units_without_charged: Query<Entity, (Without<Corpse>, Without<ChargedModifier>, With<Health>)>,
    _units_with_wet: Query<Entity, With<WetModifier>>,
    units_with_cold: Query<Entity, With<ColdModifier>>,
    units_with_dry: Query<Entity, With<DryModifier>>,
    units_with_charged: Query<Entity, With<ChargedModifier>>,
) {
    let active = weather.active;

    // Apply Wet + Charged if storm, remove if not
    if active == Some(WeatherType::Storm) {
        for entity in units_without_wet.iter() {
            commands.entity(entity).insert(WetModifier {
                intensity: weather.intensity,
                time_remaining: super::components::WET_DURATION,
            });
        }
        for entity in units_without_charged.iter() {
            commands.entity(entity).insert(ChargedModifier {
                intensity: weather.intensity,
            });
        }
    } else {
        // Don't remove wet — let the timer expire naturally (10s duration).
        // Only remove Charged immediately (storm-exclusive effect).
        for entity in units_with_charged.iter() {
            commands.entity(entity).remove::<ChargedModifier>();
        }
    }

    // Apply Cold if blizzard, remove if not
    if active == Some(WeatherType::Blizzard) {
        for entity in units_without_cold.iter() {
            commands.entity(entity).insert(ColdModifier {
                intensity: weather.intensity,
            });
        }
    } else {
        for entity in units_with_cold.iter() {
            commands.entity(entity).remove::<ColdModifier>();
        }
    }

    // Apply Dry if drought, remove if not
    if active == Some(WeatherType::Drought) {
        for entity in units_without_dry.iter() {
            commands.entity(entity).insert(DryModifier {
                intensity: weather.intensity,
            });
        }
    } else {
        for entity in units_with_dry.iter() {
            commands.entity(entity).remove::<DryModifier>();
        }
    }
}

/// Syncs intensity value to existing weather status components.
pub fn update_weather_intensity(
    weather: Res<WeatherState>,
    mut wet_query: Query<&mut WetModifier>,
    mut cold_query: Query<&mut ColdModifier>,
    mut dry_query: Query<&mut DryModifier>,
    mut charged_query: Query<&mut ChargedModifier>,
) {
    let intensity = weather.intensity;
    match weather.active {
        Some(WeatherType::Storm) => {
            for mut m in wet_query.iter_mut() {
                m.intensity = intensity;
                // Refresh timer while storm is active
                m.time_remaining = super::components::WET_DURATION;
            }
            for mut m in charged_query.iter_mut() {
                m.intensity = intensity;
            }
        }
        Some(WeatherType::Blizzard) => {
            for mut m in cold_query.iter_mut() {
                m.intensity = intensity;
            }
        }
        Some(WeatherType::Drought) => {
            for mut m in dry_query.iter_mut() {
                m.intensity = intensity;
            }
        }
        None => {}
    }
}

/// Spreads ElectricCharge from shocked units to nearby wet units.
/// Works for any source of Wet (ponds or storm weather).
pub fn spread_shock_to_wet(
    mut commands: Commands,
    weather: Option<Res<WeatherState>>,
    shocked_wet: Query<(&Transform, &ElectricCharge, &WetModifier), Without<Corpse>>,
    wet_targets: Query<
        (Entity, &Transform, Has<ElectricCharge>, Has<SpellShield>),
        (With<WetModifier>, Without<Corpse>),
    >,
) {
    // Use weather intensity for spread radius if storm is active, otherwise base radius
    let intensity = weather
        .as_ref()
        .filter(|w| w.active == Some(WeatherType::Storm))
        .map(|w| w.intensity)
        .unwrap_or(1.0);
    let spread_radius = WET_SHOCK_SPREAD_RADIUS * intensity;

    for (source_tf, charge, _wet) in shocked_wet.iter() {
        let source_pos = source_tf.translation;

        for (target_entity, target_tf, already_shocked, has_shield) in wet_targets.iter() {
            if already_shocked || has_shield {
                continue;
            }
            let dx = source_pos.x - target_tf.translation.x;
            let dz = source_pos.z - target_tf.translation.z;
            let dist_sq = dx * dx + dz * dz;

            if dist_sq <= spread_radius * spread_radius && dist_sq > 0.1 {
                // Spread a weaker charge (half the source's arc chance)
                let mut new_charge = ElectricCharge::new(0.0);
                new_charge.arc_chance = charge.arc_chance * 0.5;
                commands.entity(target_entity).insert(new_charge);
            }
        }
    }
}

/// Storm periodic random lightning strikes.
#[allow(clippy::too_many_arguments)]
pub fn storm_lightning(
    mut commands: Commands,
    time: Res<Time>,
    mut weather: ResMut<WeatherState>,
    game_config: Res<GameConfig>,
    sfx: Res<SpellSfxAssets>,
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
    let mut rng = rand::thread_rng();
    let target_index = rng.gen_range(0..target_count);

    let Some((entity, transform, mut health, temp_hp, has_shield)) =
        targets.iter_mut().nth(target_index)
    else {
        return;
    };

    let strike_pos = transform.translation;

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

// ---------------------------------------------------------------------------
// Weather visual effects
// ---------------------------------------------------------------------------

/// Spawns the weather overlay UI node (fullscreen tint) on entering gameplay.
pub fn spawn_weather_overlays(mut commands: Commands) {
    // Fullscreen UI overlay for sky tinting
    commands.spawn((
        WeatherOverlay,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::BLACK.with_alpha(0.0)),
        GlobalZIndex(999), // Just below brightness overlay
        Pickable::IGNORE,
        OnGameplayScreen,
    ));
}

/// Spawns the ground overlay mesh for snow/drought ground coloring.
pub fn spawn_ground_overlay(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = Plane3d::default().mesh().size(
        crate::game::constants::BATTLEFIELD_SIZE,
        crate::game::constants::BATTLEFIELD_SIZE,
    );

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE.with_alpha(0.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.5, 0.0), // Slightly above ground plane
        WeatherGroundOverlay,
        OnGameplayScreen,
    ));
}

/// Updates weather overlay tint based on active weather and intensity.
pub fn update_weather_overlay(
    weather: Res<WeatherState>,
    mut overlay: Query<&mut BackgroundColor, With<WeatherOverlay>>,
) {
    let Ok(mut bg) = overlay.single_mut() else {
        return;
    };

    let normalized = ((weather.intensity - 1.0) / 0.5).clamp(0.0, 1.0);

    match weather.active {
        Some(WeatherType::Storm) => {
            // Dark blue-gray tint with slight purple for lightning
            let alpha = RAIN_SKY_DARKEN_ALPHA * 1.1 * normalized;
            *bg = BackgroundColor(Color::srgba(0.05, 0.04, 0.14, alpha));
        }
        Some(WeatherType::Drought) => {
            // Yellowish/orange heat haze
            let alpha = DROUGHT_TINT_ALPHA * normalized;
            *bg = BackgroundColor(Color::srgba(0.6, 0.4, 0.1, alpha));
        }
        Some(WeatherType::Blizzard) => {
            // Slight white haze
            let alpha = RAIN_SKY_DARKEN_ALPHA * 0.5 * normalized;
            *bg = BackgroundColor(Color::srgba(0.7, 0.75, 0.85, alpha));
        }
        None => {
            *bg = BackgroundColor(Color::BLACK.with_alpha(0.0));
        }
    }
}

/// Updates ground overlay color for blizzard (whitening) and drought (browning).
pub fn update_ground_overlay(
    weather: Res<WeatherState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    overlay: Query<&MeshMaterial3d<StandardMaterial>, With<WeatherGroundOverlay>>,
) {
    let Ok(mat_handle) = overlay.single() else {
        return;
    };
    let Some(material) = materials.get_mut(&mat_handle.0) else {
        return;
    };

    let normalized = ((weather.intensity - 1.0) / 0.5).clamp(0.0, 1.0);

    match weather.active {
        Some(WeatherType::Blizzard) => {
            let alpha = BLIZZARD_GROUND_ALPHA * normalized;
            material.base_color = Color::srgba(0.85, 0.88, 0.95, alpha);
        }
        Some(WeatherType::Drought) => {
            let alpha = DROUGHT_GROUND_ALPHA * normalized;
            material.base_color = Color::srgba(0.55, 0.4, 0.2, alpha);
        }
        _ => {
            material.base_color = Color::WHITE.with_alpha(0.0);
        }
    }
}

/// Spawns weather particles each frame based on active weather.
pub fn spawn_weather_particles(
    mut commands: Commands,
    weather: Res<WeatherState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(weather_type) = weather.active else {
        return;
    };

    let mut rng = rand::thread_rng();
    let normalized = ((weather.intensity - 1.0) / 0.5).clamp(0.0, 1.0);
    // Scale particle count with intensity ramp (start with half, ramp to full)
    let intensity_scale = 0.5 + 0.5 * normalized;

    match weather_type {
        WeatherType::Storm => {
            let count = (RAIN_PARTICLES_PER_FRAME as f32 * intensity_scale) as u32;
            let mesh = meshes.add(Rectangle::new(1.5, 25.0));
            let mat = materials.add(StandardMaterial {
                base_color: RAIN_PARTICLE_COLOR,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });

            for _ in 0..count {
                let x = rng.gen_range(-PARTICLE_SPAWN_HALF_SIZE..PARTICLE_SPAWN_HALF_SIZE);
                let z = rng.gen_range(-PARTICLE_SPAWN_HALF_SIZE..PARTICLE_SPAWN_HALF_SIZE);
                let pos = Vec3::new(x, PARTICLE_SPAWN_HEIGHT, z);

                commands.spawn((
                    WeatherParticle {
                        velocity: Vec3::new(RAIN_WIND_SPEED, -RAIN_FALL_SPEED, 0.0),
                        lifetime: RAIN_PARTICLE_LIFETIME,
                    },
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_translation(pos).with_rotation(Quat::from_rotation_z(-0.2)),
                    OnGameplayScreen,
                ));
            }
        }
        WeatherType::Blizzard => {
            let count = (SNOW_PARTICLES_PER_FRAME as f32 * intensity_scale) as u32;
            let mesh = meshes.add(Circle::new(4.0));
            let mat = materials.add(StandardMaterial {
                base_color: SNOW_PARTICLE_COLOR,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });

            for _ in 0..count {
                let x = rng.gen_range(-PARTICLE_SPAWN_HALF_SIZE..PARTICLE_SPAWN_HALF_SIZE);
                let z = rng.gen_range(-PARTICLE_SPAWN_HALF_SIZE..PARTICLE_SPAWN_HALF_SIZE);
                let pos = Vec3::new(x, PARTICLE_SPAWN_HEIGHT, z);

                // Snow drifts more horizontally with random wobble
                let vx = SNOW_WIND_SPEED + rng.gen_range(-200.0..200.0);
                let vz = rng.gen_range(-300.0..300.0);

                commands.spawn((
                    WeatherParticle {
                        velocity: Vec3::new(vx, -SNOW_FALL_SPEED, vz),
                        lifetime: SNOW_PARTICLE_LIFETIME,
                    },
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_translation(pos),
                    OnGameplayScreen,
                ));
            }
        }
        WeatherType::Drought => {
            // Drought has no falling particles — uses overlay tint and ground browning only
        }
    }
}

/// Moves and despawns weather particles.
pub fn update_weather_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut WeatherParticle, &mut Transform)>,
) {
    let delta = time.delta_secs();
    for (entity, mut particle, mut transform) in particles.iter_mut() {
        particle.lifetime -= delta;
        if particle.lifetime <= 0.0 || transform.translation.y <= 0.0 {
            commands.entity(entity).try_despawn();
            continue;
        }
        transform.translation += particle.velocity * delta;
    }
}
