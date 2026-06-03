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

    // Particles are purely visual. Use a NON-seeded RNG so running this on both
    // multiplayer peers (for full-fidelity weather visuals) doesn't perturb the
    // shared seeded `GameRng` stream and desync gameplay.
    let mut thread_rng = rand::rng();
    let rng = &mut thread_rng;
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
            let mesh = meshes.add(Circle::new(4.0));
            let mat = materials.add(StandardMaterial {
                base_color: SNOW_PARTICLE_COLOR,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });

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
