//! Weather visuals: overlays and particle systems.

use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use super::resources::{WeatherState, WeatherType};
use crate::game::components::OnGameplayScreen;

/// Applies the Drought healing reduction to a heal amount.
/// Returns the (possibly reduced) heal amount.
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

/// Alpha-composites `top` over `base` (standard "src over"). Lets two weather
/// tints stack into one overlay so both opposing weathers show at once.
fn composite_over(base: Srgba, top: Srgba) -> Srgba {
    let a = top.alpha + base.alpha * (1.0 - top.alpha);
    if a <= 0.0 {
        return Srgba::new(0.0, 0.0, 0.0, 0.0);
    }
    Srgba::new(
        (top.red * top.alpha + base.red * base.alpha * (1.0 - top.alpha)) / a,
        (top.green * top.alpha + base.green * base.alpha * (1.0 - top.alpha)) / a,
        (top.blue * top.alpha + base.blue * base.alpha * (1.0 - top.alpha)) / a,
        a,
    )
}

/// Sky-overlay tint for one weather at a given intensity.
fn overlay_tint(weather: WeatherType, intensity: f32) -> Srgba {
    let normalized = ((intensity - 1.0) / 0.5).clamp(0.0, 1.0);
    match weather {
        // Dark blue-gray tint with slight purple for lightning
        WeatherType::Storm => {
            Srgba::new(0.05, 0.04, 0.14, RAIN_SKY_DARKEN_ALPHA * 1.1 * normalized)
        }
        // Yellowish/orange heat haze
        WeatherType::Drought => Srgba::new(0.6, 0.4, 0.1, DROUGHT_TINT_ALPHA * normalized),
        // Slight white haze
        WeatherType::Blizzard => {
            Srgba::new(0.7, 0.75, 0.85, RAIN_SKY_DARKEN_ALPHA * 0.5 * normalized)
        }
    }
}

/// Updates weather overlay tint, blending BOTH active slots' weather.
pub fn update_weather_overlay(
    weather: Res<WeatherState>,
    mut overlay: Query<&mut BackgroundColor, With<WeatherOverlay>>,
) {
    let Ok(mut bg) = overlay.single_mut() else {
        return;
    };

    let mut result = Srgba::new(0.0, 0.0, 0.0, 0.0);
    for slot in [&weather.local, &weather.remote] {
        if let Some(w) = slot.active {
            result = composite_over(result, overlay_tint(w, slot.intensity));
        }
    }
    *bg = BackgroundColor(result.into());
}

/// Ground-overlay tint for one weather (Storm has none).
fn ground_tint(weather: WeatherType, intensity: f32) -> Option<Srgba> {
    let normalized = ((intensity - 1.0) / 0.5).clamp(0.0, 1.0);
    match weather {
        WeatherType::Blizzard => Some(Srgba::new(
            0.85,
            0.88,
            0.95,
            BLIZZARD_GROUND_ALPHA * normalized,
        )),
        WeatherType::Drought => Some(Srgba::new(
            0.55,
            0.4,
            0.2,
            DROUGHT_GROUND_ALPHA * normalized,
        )),
        WeatherType::Storm => None,
    }
}

/// Updates ground overlay color, blending blizzard (whitening) and drought
/// (browning) across BOTH active slots.
pub fn update_ground_overlay(
    weather: Res<WeatherState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    overlay: Query<&MeshMaterial3d<StandardMaterial>, With<WeatherGroundOverlay>>,
) {
    let Ok(mat_handle) = overlay.single() else {
        return;
    };
    let Some(mut material) = materials.get_mut(&mat_handle.0) else {
        return;
    };

    let mut result = Srgba::new(0.0, 0.0, 0.0, 0.0);
    for slot in [&weather.local, &weather.remote] {
        if let Some(tint) = slot.active.and_then(|w| ground_tint(w, slot.intensity)) {
            result = composite_over(result, tint);
        }
    }
    material.base_color = result.into();
}

/// Spawns weather particles each frame based on active weather.
#[allow(clippy::too_many_arguments)]
pub fn spawn_weather_particles(
    mut commands: Commands,
    weather: Res<WeatherState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // The particle mesh/material are invariant — build each once and reuse the
    // handle so we don't allocate a fresh asset entry every frame. The `Local`s
    // keep the handles alive for the session.
    mut rain_mesh: Local<Option<Handle<Mesh>>>,
    mut rain_mat: Local<Option<Handle<StandardMaterial>>>,
    mut snow_mesh: Local<Option<Handle<Mesh>>>,
    mut snow_mat: Local<Option<Handle<StandardMaterial>>>,
) {
    // Particles are purely visual. Use a NON-seeded RNG so running this on both
    // multiplayer peers (for full-fidelity weather visuals) doesn't perturb the
    // shared seeded `GameRng` stream and desync gameplay.
    let mut thread_rng = rand::rng();
    let rng = &mut thread_rng;

    // Render particles for BOTH slots so two opposing weathers both show
    // (e.g. one player's rain alongside the other's snow).
    for slot in [&weather.local, &weather.remote] {
        let Some(weather_type) = slot.active else {
            continue;
        };

        let normalized = ((slot.intensity - 1.0) / 0.5).clamp(0.0, 1.0);
        // Scale particle count with intensity ramp (start with half, ramp to full)
        let intensity_scale = 0.5 + 0.5 * normalized;

        match weather_type {
            WeatherType::Storm => {
                let count = (RAIN_PARTICLES_PER_FRAME as f32 * intensity_scale) as u32;
                let mesh = rain_mesh
                    .get_or_insert_with(|| meshes.add(Rectangle::new(1.5, 25.0)))
                    .clone();
                let mat = rain_mat
                    .get_or_insert_with(|| {
                        materials.add(StandardMaterial {
                            base_color: RAIN_PARTICLE_COLOR,
                            alpha_mode: AlphaMode::Blend,
                            unlit: true,
                            // Double-sided: the multiplayer guest's camera is mirrored to
                            // the far side of the battlefield and would otherwise see only
                            // the backface of these flat quads (culled by default).
                            cull_mode: None,
                            ..default()
                        })
                    })
                    .clone();

                for _ in 0..count {
                    let x = rng.random_range(-PARTICLE_SPAWN_HALF_SIZE..PARTICLE_SPAWN_HALF_SIZE);
                    let z = rng.random_range(-PARTICLE_SPAWN_HALF_SIZE..PARTICLE_SPAWN_HALF_SIZE);
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
                let mesh = snow_mesh
                    .get_or_insert_with(|| meshes.add(Circle::new(4.0)))
                    .clone();
                let mat = snow_mat
                    .get_or_insert_with(|| {
                        materials.add(StandardMaterial {
                            base_color: SNOW_PARTICLE_COLOR,
                            alpha_mode: AlphaMode::Blend,
                            unlit: true,
                            // Double-sided so the mirrored multiplayer guest camera sees them.
                            cull_mode: None,
                            ..default()
                        })
                    })
                    .clone();

                for _ in 0..count {
                    let x = rng.random_range(-PARTICLE_SPAWN_HALF_SIZE..PARTICLE_SPAWN_HALF_SIZE);
                    let z = rng.random_range(-PARTICLE_SPAWN_HALF_SIZE..PARTICLE_SPAWN_HALF_SIZE);
                    let pos = Vec3::new(x, PARTICLE_SPAWN_HEIGHT, z);

                    // Snow drifts more horizontally with random wobble
                    let vx = SNOW_WIND_SPEED + rng.random_range(-200.0..200.0);
                    let vz = rng.random_range(-300.0..300.0);

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
