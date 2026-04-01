use bevy::prelude::*;

use super::components::Tree;
use super::constants::*;
use super::resources::TreeAssets;
use crate::game::components::{Billboard, ObstacleHealth, OnGameplayScreen};
use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::shared_systems::{ShadowAssets, spawn_terrain_shadow};

/// Spawns a single tree at the given position with a size multiplier.
pub(in crate::game) fn spawn_single_tree(
    commands: &mut Commands,
    assets: &TreeAssets,
    shadow_assets: &ShadowAssets,
    x: f32,
    z: f32,
    scale: f32,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) {
    let center = Vec3::new(x, 0.0, z);
    let radius = TREE_RADIUS * scale;
    let height = TREE_HEIGHT * scale;

    commands.spawn((
        Mesh3d(assets.mesh.clone()),
        MeshMaterial3d(assets.material.clone()),
        Transform::from_xyz(x, TREE_VISUAL_HEIGHT * scale / 2.0 + 1.0, z)
            .with_scale(Vec3::splat(scale)),
        Tree {
            center,
            radius,
            height,
        },
        ObstacleHealth::new(TREE_HEALTH * scale),
        Billboard,
        OnGameplayScreen,
    ));

    spawn_terrain_shadow(commands, shadow_assets, x, z, 1.2 * scale);

    let center_xz = Vec2::new(x, z);
    obstacle_events.write(ObstacleChanged {
        bounds: Rect::from_center_half_size(center_xz, Vec2::splat(radius)),
        obstacle_type: ObstacleType::Blocked,
        shape: Some(ObstacleShape::circle(center_xz, radius)),
        rebuild: false,
    });
}

/// Destroys trees that have lost all health.
pub fn destroy_dead_trees(
    mut commands: Commands,
    trees: Query<(Entity, &Tree, &ObstacleHealth)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, tree, health) in &trees {
        if health.is_dead() {
            // Remove from pathfinding
            let center_xz = Vec2::new(tree.center.x, tree.center.z);
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_half_size(center_xz, Vec2::splat(tree.radius)),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::circle(center_xz, tree.radius)),
                rebuild: false,
            });

            commands.entity(entity).try_despawn();
        }
    }
}

/// Applies fire spell AoE damage to trees (only fire damage destroys them).
pub fn apply_spell_damage_to_trees(
    mut trees: Query<(&Tree, &mut ObstacleHealth)>,
    explosions: Query<&crate::game::units::wizard::spells::fireball::components::FireballExplosion>,
    meteor_explosions: Query<
        &crate::game::units::wizard::spells::meteor_fall::components::MeteorExplosion,
    >,
) {
    let xz_distance = |a: Vec3, b: Vec3| -> f32 {
        let dx = a.x - b.x;
        let dz = a.z - b.z;
        (dx * dx + dz * dz).sqrt()
    };

    for (tree, mut health) in &mut trees {
        if health.is_dead() {
            continue;
        }

        // Fireball explosions
        for explosion in &explosions {
            if explosion.damage_per_tick > 0.0
                && xz_distance(explosion.origin, tree.center)
                    <= explosion.current_radius() + tree.radius
            {
                health.take_damage(explosion.damage_per_tick);
            }
        }

        // Meteor explosions
        for explosion in &meteor_explosions {
            if !explosion.damage_applied
                && xz_distance(explosion.origin, tree.center) <= explosion.max_radius + tree.radius
            {
                health.take_damage(explosion.damage);
            }
        }
    }
}
