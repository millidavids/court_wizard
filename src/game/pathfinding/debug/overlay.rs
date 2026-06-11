use bevy::prelude::*;

use crate::game::components::OnGameplayScreen;
use crate::game::constants::{BATTLEFIELD_SIZE, STAGING_POINTS};

// ===== Debug Ball Constants =====

/// Movement increment per arrow key press (world units).
const DEBUG_BALL_STEP: f32 = 5.0;
/// Large movement increment per Y/G/H/J key press (world units).
const DEBUG_BALL_LARGE_STEP: f32 = 100.0;
/// Radius of the debug ball mesh.
const DEBUG_BALL_RADIUS: f32 = 20.0;
/// Height above the battlefield (Y coordinate).
const DEBUG_BALL_Y: f32 = 20.0;
/// How often to log the ball's position (seconds).
const DEBUG_BALL_LOG_INTERVAL: f32 = 5.0;

/// Whether the debug ball mode is active.
#[derive(Resource, Default)]
pub(crate) struct DebugBallActive(pub(crate) bool);

/// Marker component for the debug ball entity.
#[derive(Component)]
pub(crate) struct DebugBall;

/// Marker component for staging point debug markers (red balls).
#[derive(Component)]
pub(crate) struct StagingPointMarker;

/// Timer for periodic position logging.
#[derive(Resource)]
pub(crate) struct DebugBallLogTimer(pub(crate) Timer);

impl Default for DebugBallLogTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            DEBUG_BALL_LOG_INTERVAL,
            TimerMode::Repeating,
        ))
    }
}

/// Toggles the debug ball on F4. Spawns or despawns the ball entity
/// and red staging point markers.
#[allow(clippy::too_many_arguments)]
pub(crate) fn toggle_debug_ball(
    keys: Res<ButtonInput<KeyCode>>,
    mut active: ResMut<DebugBallActive>,
    mut commands: Commands,
    ball_query: Query<Entity, With<DebugBall>>,
    staging_marker_query: Query<Entity, With<StagingPointMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut log_timer: ResMut<DebugBallLogTimer>,
) {
    if !keys.just_pressed(KeyCode::F4) {
        return;
    }

    active.0 = !active.0;

    if active.0 {
        info!(
            "Debug ball: ON — use arrow keys to move, position logged every {DEBUG_BALL_LOG_INTERVAL}s"
        );
        log_timer.0.reset();

        let mesh = meshes.add(Sphere::new(DEBUG_BALL_RADIUS));
        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        });

        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, DEBUG_BALL_Y, 0.0),
            DebugBall,
            OnGameplayScreen,
        ));

        // Spawn red markers at each staging point
        let staging_mesh = meshes.add(Sphere::new(DEBUG_BALL_RADIUS));
        let staging_material = materials.add(StandardMaterial {
            base_color: Color::linear_rgb(1.0, 0.0, 0.0),
            unlit: true,
            ..default()
        });
        for &(sx, sz) in &STAGING_POINTS {
            commands.spawn((
                Mesh3d(staging_mesh.clone()),
                MeshMaterial3d(staging_material.clone()),
                Transform::from_xyz(sx, DEBUG_BALL_Y, sz),
                StagingPointMarker,
                OnGameplayScreen,
            ));
        }

        info!("Debug ball position: X=0.0, Z=0.0");
    } else {
        info!("Debug ball: OFF");
        for entity in &ball_query {
            commands.entity(entity).try_despawn();
        }
        for entity in &staging_marker_query {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Moves the debug ball with arrow keys in 5-unit increments, clamped to battlefield bounds.
pub(crate) fn move_debug_ball(
    keys: Res<ButtonInput<KeyCode>>,
    active: Res<DebugBallActive>,
    mut ball_query: Query<&mut Transform, With<DebugBall>>,
) {
    if !active.0 {
        return;
    }

    let Ok(mut transform) = ball_query.single_mut() else {
        return;
    };

    let half = BATTLEFIELD_SIZE / 2.0;

    if keys.just_pressed(KeyCode::ArrowRight) {
        transform.translation.x = (transform.translation.x + DEBUG_BALL_STEP).min(half);
    }
    if keys.just_pressed(KeyCode::ArrowLeft) {
        transform.translation.x = (transform.translation.x - DEBUG_BALL_STEP).max(-half);
    }
    // Arrow up moves toward -Z (further from camera in typical view)
    if keys.just_pressed(KeyCode::ArrowUp) {
        transform.translation.z = (transform.translation.z - DEBUG_BALL_STEP).max(-half);
    }
    // Arrow down moves toward +Z (closer to camera)
    if keys.just_pressed(KeyCode::ArrowDown) {
        transform.translation.z = (transform.translation.z + DEBUG_BALL_STEP).min(half);
    }

    // Large movement: Y (up/-Z), H (down/+Z), G (left/-X), J (right/+X)
    if keys.just_pressed(KeyCode::KeyJ) {
        transform.translation.x = (transform.translation.x + DEBUG_BALL_LARGE_STEP).min(half);
    }
    if keys.just_pressed(KeyCode::KeyG) {
        transform.translation.x = (transform.translation.x - DEBUG_BALL_LARGE_STEP).max(-half);
    }
    if keys.just_pressed(KeyCode::KeyY) {
        transform.translation.z = (transform.translation.z - DEBUG_BALL_LARGE_STEP).max(-half);
    }
    if keys.just_pressed(KeyCode::KeyH) {
        transform.translation.z = (transform.translation.z + DEBUG_BALL_LARGE_STEP).min(half);
    }
}

/// Logs the debug ball position every 5 seconds while active.
pub(crate) fn log_debug_ball_position(
    time: Res<Time>,
    active: Res<DebugBallActive>,
    mut log_timer: ResMut<DebugBallLogTimer>,
    ball_query: Query<&Transform, With<DebugBall>>,
) {
    if !active.0 {
        return;
    }

    log_timer.0.tick(time.delta());

    if !log_timer.0.just_finished() {
        return;
    }

    let Ok(transform) = ball_query.single() else {
        return;
    };

    info!(
        "Debug ball position: X={:.1}, Z={:.1}",
        transform.translation.x, transform.translation.z
    );
}
