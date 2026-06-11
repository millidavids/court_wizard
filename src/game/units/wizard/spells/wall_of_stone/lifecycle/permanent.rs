use super::super::components::{WallHealth, WallOfStone};
use super::super::constants::*;
use crate::config::save_data::SavedWall;
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

/// Spawns a permanent wall entity from saved wall data.
pub(crate) fn spawn_permanent_wall(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    saved: &SavedWall,
) {
    let forward = Vec3::new(saved.forward_x, 0.0, saved.forward_z);
    let right = Vec3::new(-forward.z, 0.0, forward.x);
    let center = Vec3::new(saved.center_x, 0.0, saved.center_z);
    let rotation = Quat::from_rotation_arc(Vec3::X, forward);

    commands.spawn((
        Mesh3d(assets.unit_cuboid.clone()),
        MeshMaterial3d(assets.wall_of_stone.clone()),
        Transform::from_xyz(center.x, saved.height / 2.0, center.z)
            .with_rotation(rotation)
            .with_scale(Vec3::new(
                saved.half_length * 2.0,
                saved.height,
                saved.half_width * 2.0,
            )),
        WallOfStone {
            center,
            half_length: saved.half_length,
            half_width: saved.half_width,
            forward,
            right,
            height: saved.height,
            time_alive: 0.0,
            duration: f32::MAX,
            sinking: false,
            empowerment: saved.empowerment,
            permanent: true,
        },
        WallHealth::new(WALL_HEALTH),
        NetworkedSpellEffect {
            kind: SpellEffectKind::WallOfStone,
        },
        OnGameplayScreen,
    ));
}

/// Registers pathfinding obstacles for all permanent walls after loading completes.
pub(crate) fn register_permanent_wall_obstacles(
    walls: Query<&WallOfStone>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for wall in &walls {
        if !wall.permanent {
            continue;
        }
        let obs_bounds = wall.obstacle_bounds();
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
            obstacle_type: ObstacleType::Blocked,
            shape: Some(ObstacleShape::obb_from_center(
                wall.center,
                wall.forward,
                wall.half_length,
                wall.half_width,
            )),
            rebuild: false,
        });
    }
}
