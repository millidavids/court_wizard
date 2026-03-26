//! Flow field debug visualization.
//!
//! Press F3 to cycle: Off → Attacker → Defender → Off.
//! Renders white arrows above the battlefield showing flow field directions.
//!
//! Press F4 to toggle a moveable debug ball for pinpointing world positions.
//! Arrow keys move it in 5-unit increments on the XZ plane.
//! Position is logged every 5 seconds while active.

use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::game::components::OnGameplayScreen;
use crate::game::constants::BATTLEFIELD_SIZE;

use super::resources::PathfindingGrid;

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

/// Which flow field is being visualized.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum FlowFieldDebugMode {
    #[default]
    Off,
    Attacker,
    Defender,
}

impl FlowFieldDebugMode {
    pub(super) fn next(self) -> Self {
        match self {
            Self::Off => Self::Attacker,
            Self::Attacker => Self::Defender,
            Self::Defender => Self::Off,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Attacker => "Attacker",
            Self::Defender => "Defender",
        }
    }
}

/// Marker component for debug arrow entities (bulk despawn).
#[derive(Component)]
pub(super) struct FlowFieldDebugArrow;

/// Shared mesh + material handles allocated once on first use.
#[derive(Resource)]
pub(super) struct DebugArrowAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

/// Cycles debug mode on F3.
pub(super) fn toggle_flow_field_debug(
    keys: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<FlowFieldDebugMode>,
) {
    if keys.just_pressed(KeyCode::F3) {
        let next = mode.next();
        info!("Flow field debug: {}", next.label());
        *mode = next;
    }
}

/// Spawns/despawns debug arrows when mode or field data changes.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_debug_visualization(
    mut commands: Commands,
    mode: Res<FlowFieldDebugMode>,
    pathfinding: Res<PathfindingGrid>,
    arrows: Query<Entity, With<FlowFieldDebugArrow>>,
    mut prev_mode: Local<FlowFieldDebugMode>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Option<Res<DebugArrowAssets>>,
) {
    let mode_changed = *mode != *prev_mode;
    // Bevy's change detection: true when PathfindingGrid was mutably accessed.
    let field_changed = *mode != FlowFieldDebugMode::Off && pathfinding.is_changed();

    *prev_mode = *mode;

    if !mode_changed && !field_changed {
        return;
    }

    // Despawn existing arrows.
    for entity in &arrows {
        commands.entity(entity).try_despawn();
    }

    if *mode == FlowFieldDebugMode::Off {
        return;
    }

    // Pick the appropriate field.
    let field = match *mode {
        FlowFieldDebugMode::Attacker => pathfinding.attacker_field.as_ref(),
        FlowFieldDebugMode::Defender => pathfinding.defender_field.as_ref(),
        FlowFieldDebugMode::Off => unreachable!(),
    };

    let field = match field {
        Some(f) => f,
        None => return,
    };

    let cell_size = pathfinding.cell_size;
    let world_min = pathfinding.world_min;

    // Initialize shared handles once.
    let (mesh_handle, material_handle) = if let Some(res) = &assets {
        (res.mesh.clone(), res.material.clone())
    } else {
        // Triangle in XZ plane: 10 long (+Z), 4 wide (X), normal up (+Y).
        // Tip points along +Z; rotated around Y at spawn time.
        let mut tri = Mesh::new(PrimitiveTopology::TriangleList, default());
        tri.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [0.0, 0.0, 12.5],   // tip (forward)
                [-2.0, 0.0, -12.5], // base left
                [2.0, 0.0, -12.5],  // base right
            ],
        );
        tri.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
        );
        tri.insert_indices(Indices::U32(vec![0, 1, 2]));
        let mesh = meshes.add(tri);
        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        commands.insert_resource(DebugArrowAssets {
            mesh: mesh.clone(),
            material: material.clone(),
        });
        (mesh, material)
    };
    let stride = 3usize;

    for z in (0..field.height).step_by(stride) {
        for x in (0..field.width).step_by(stride) {
            let idx = z * field.width + x;
            let dir = field.directions[idx];

            if dir == Vec3::ZERO {
                continue;
            }

            let world_x = world_min.x + (x as f32 + 0.5) * cell_size;
            let world_z = world_min.y + (z as f32 + 0.5) * cell_size;

            // Mesh tip points along +Z; rotate around Y to match flow direction.
            let angle = dir.x.atan2(dir.z);
            commands.spawn((
                Mesh3d(mesh_handle.clone()),
                MeshMaterial3d(material_handle.clone()),
                Transform::from_xyz(world_x, 5.0, world_z)
                    .with_rotation(Quat::from_rotation_y(angle)),
                FlowFieldDebugArrow,
                OnGameplayScreen,
            ));
        }
    }
}

// ===== F4 Debug Ball =====

/// Whether the debug ball mode is active.
#[derive(Resource, Default)]
pub(super) struct DebugBallActive(pub bool);

/// Marker component for the debug ball entity.
#[derive(Component)]
pub(super) struct DebugBall;

/// Timer for periodic position logging.
#[derive(Resource)]
pub(super) struct DebugBallLogTimer(Timer);

impl Default for DebugBallLogTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            DEBUG_BALL_LOG_INTERVAL,
            TimerMode::Repeating,
        ))
    }
}

/// Toggles the debug ball on F4. Spawns or despawns the ball entity.
pub(super) fn toggle_debug_ball(
    keys: Res<ButtonInput<KeyCode>>,
    mut active: ResMut<DebugBallActive>,
    mut commands: Commands,
    ball_query: Query<Entity, With<DebugBall>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut log_timer: ResMut<DebugBallLogTimer>,
) {
    if !keys.just_pressed(KeyCode::F4) {
        return;
    }

    active.0 = !active.0;

    if active.0 {
        info!("Debug ball: ON — use arrow keys to move, position logged every {DEBUG_BALL_LOG_INTERVAL}s");
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

        info!("Debug ball position: X=0.0, Z=0.0");
    } else {
        info!("Debug ball: OFF");
        for entity in &ball_query {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Moves the debug ball with arrow keys in 5-unit increments, clamped to battlefield bounds.
pub(super) fn move_debug_ball(
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
pub(super) fn log_debug_ball_position(
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
