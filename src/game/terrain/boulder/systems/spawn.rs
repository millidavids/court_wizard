use bevy::prelude::*;

use super::super::components::{Boulder, BoulderShadow};
use super::super::constants::*;
use super::super::resources::BoulderAssets;
use crate::game::components::{Billboard, ObstacleHealth, OnGameplayScreen};
use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::shared_systems::ShadowAssets;
use crate::game::units::components::Teleportable;

/// Spawns a pre-placed terrain boulder (identical to a landed thrown boulder).
/// Used by the terrain generation system, not by brute/ogre throws.
#[allow(clippy::too_many_arguments)]
pub(in crate::game) fn spawn_terrain_boulder(
    commands: &mut Commands,
    rock_assets: &BoulderAssets,
    shadow_assets: &ShadowAssets,
    x: f32,
    z: f32,
    scale: f32,
    sprite_index: u8,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) {
    let radius = ROCK_RADIUS * scale;
    let rock_y = BOULDER_SPRITE_HEIGHT * scale / 2.0 - BOULDER_GROUND_CLIP;
    let idx = (sprite_index as usize).min(BOULDER_SPRITE_COUNT - 1);

    let rock = Boulder {
        center: Vec3::new(x, 0.0, z),
        radius,
        height: ROCK_HEIGHT * scale,
        sinking: false,
        time_alive: 0.0,
        sink_deadline: f32::MAX,
        sprite_index,
    };

    let center_xz = Vec2::new(x, z);
    obstacle_events.write(ObstacleChanged {
        bounds: Rect::from_center_half_size(center_xz, Vec2::splat(radius)),
        obstacle_type: ObstacleType::Blocked,
        shape: Some(ObstacleShape::circle(center_xz, radius)),
        rebuild: false,
    });

    let rock_entity = commands
        .spawn((
            Mesh3d(rock_assets.mesh.clone()),
            MeshMaterial3d(rock_assets.materials[idx].clone()),
            Transform::from_xyz(x, rock_y, z).with_scale(Vec3::splat(scale)),
            rock,
            ObstacleHealth::new(ROCK_HEALTH * scale),
            Billboard,
            Teleportable,
            // Tag pre-placed terrain boulders too so the guest sees them
            // via the snapshot pipeline. Without this only dynamically
            // thrown boulders (brute/ogre) would be visible on the guest.
            crate::game::multiplayer::components::NetworkedSpellEffect {
                kind: crate::networking::snapshot::SpellEffectKind::BoulderObstacle,
            },
            OnGameplayScreen,
        ))
        .id();

    commands.spawn((
        Mesh3d(shadow_assets.mesh.clone()),
        MeshMaterial3d(shadow_assets.material.clone()),
        Transform::from_xyz(x, ROCK_SHADOW_Y, z)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(ROCK_SHADOW_SCALE * scale)),
        BoulderShadow { owner: rock_entity },
        OnGameplayScreen,
    ));
}
