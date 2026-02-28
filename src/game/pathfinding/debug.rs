//! Flow field debug visualization.
//!
//! Press F3 to cycle: Off → Attacker → Defender → Off.
//! Renders white arrows above the battlefield showing flow field directions.

use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::game::components::OnGameplayScreen;

use super::resources::PathfindingGrid;

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
        commands.entity(entity).despawn();
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
        tri.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![
            [0.0, 0.0, 12.5],   // tip (forward)
            [-2.0, 0.0, -12.5], // base left
            [2.0, 0.0, -12.5],  // base right
        ]);
        tri.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]);
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
