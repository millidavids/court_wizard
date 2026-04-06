use bevy::prelude::*;

use super::components::{BurningTree, Tree};
use super::constants::*;
use super::resources::TreeAssets;
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::shared_systems::{ShadowAssets, spawn_terrain_shadow};
use crate::game::terrain::utils::{
    BURNING_VEGETATION_TINT, emit_burning_vfx, should_ignite_from_fire,
};
use crate::game::terrain::wind_sway::material::WindSwayMaterial;
use crate::game::units::components::Corpse;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Spawns a single tree at the given position with a size multiplier.
pub(in crate::game) fn spawn_single_tree(
    commands: &mut Commands,
    assets: &TreeAssets,
    shadow_assets: &ShadowAssets,
    x: f32,
    z: f32,
    scale: f32,
    sprite_index: u8,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) {
    let center = Vec3::new(x, 0.0, z);
    let radius = tree_radius_for_variant(sprite_index) * scale;
    let height = TREE_HEIGHT * scale;
    let idx = (sprite_index as usize).min(TREE_SPRITE_COUNT - 1);

    commands.spawn((
        Mesh3d(assets.mesh.clone()),
        MeshMaterial3d(assets.materials[idx].clone()),
        Transform::from_xyz(x, TREE_SPRITE_HEIGHT * scale / 2.0 - TREE_GROUND_CLIP, z)
            .with_scale(Vec3::splat(scale)),
        Tree {
            center,
            radius,
            height,
            sprite_index,
        },
        Billboard,
        OnGameplayScreen,
    ));

    spawn_terrain_shadow(commands, shadow_assets, x, z, TREE_SHADOW_SCALE * scale);

    let center_xz = Vec2::new(x, z);
    obstacle_events.write(ObstacleChanged {
        bounds: Rect::from_center_half_size(center_xz, Vec2::splat(radius)),
        obstacle_type: ObstacleType::Blocked,
        shape: Some(ObstacleShape::circle(center_xz, radius)),
        rebuild: false,
    });
}

/// Ignites trees hit by fire spell explosions or disintegrate beams.
#[allow(clippy::too_many_arguments)]
pub fn ignite_trees_from_fire(
    mut commands: Commands,
    trees: Query<(Entity, &Tree), Without<BurningTree>>,
    explosions: Query<&crate::game::units::wizard::spells::fireball::components::FireballExplosion>,
    meteor_explosions: Query<
        &crate::game::units::wizard::spells::meteor_fall::components::MeteorExplosion,
    >,
    beams: Query<&crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam>,
    tree_assets: Res<TreeAssets>,
    mut materials: ResMut<Assets<WindSwayMaterial>>,
) {
    for (entity, tree) in &trees {
        if !should_ignite_from_fire(
            tree.center,
            tree.radius,
            &explosions,
            &meteor_explosions,
            &beams,
        ) {
            continue;
        }

        commands
            .entity(entity)
            .insert(BurningTree { tick_timer: 0.0 });

        let idx = (tree.sprite_index as usize).min(TREE_SPRITE_COUNT - 1);
        if let Some(shared_mat) = materials.get(&tree_assets.materials[idx]) {
            let mut tinted = shared_mat.clone();
            tinted.base_color = BURNING_VEGETATION_TINT.to_linear();
            let new_mat = materials.add(tinted);
            commands.entity(entity).insert(MeshMaterial3d(new_mat));
        }
    }
}

/// Burning trees deal periodic fire damage to units inside them.
pub fn apply_burning_tree_damage(
    time: Res<Time>,
    mut burning_trees: Query<(&Tree, &mut BurningTree)>,
    mut units: Query<
        (
            &Transform,
            &mut crate::game::units::components::Health,
            Option<&mut crate::game::units::components::TemporaryHitPoints>,
        ),
        Without<Corpse>,
    >,
) {
    let delta = time.delta_secs();

    for (tree, mut burning) in &mut burning_trees {
        burning.tick_timer += delta;
        if burning.tick_timer < BURNING_TREE_TICK_INTERVAL {
            continue;
        }
        burning.tick_timer -= BURNING_TREE_TICK_INTERVAL;

        let damage = BURNING_TREE_DPS * BURNING_TREE_TICK_INTERVAL;

        for (transform, mut health, mut temp_hp) in &mut units {
            let dx = transform.translation.x - tree.center.x;
            let dz = transform.translation.z - tree.center.z;
            if dx * dx + dz * dz <= tree.radius * tree.radius {
                crate::game::units::components::apply_damage_to_unit(
                    &mut health,
                    temp_hp.as_deref_mut(),
                    damage,
                );
            }
        }
    }
}

/// Emits fire smoke and spark particles from burning trees.
pub fn emit_burning_tree_vfx(
    mut commands: Commands,
    burning_trees: Query<&Tree, With<BurningTree>>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut smoke_timer: Local<f32>,
    mut spark_timer: Local<f32>,
) {
    let delta = time.delta_secs();
    let t = time.elapsed_secs();

    *smoke_timer += delta;
    *spark_timer += delta;

    let emit_smoke = *smoke_timer >= BURNING_TREE_SMOKE_INTERVAL;
    let emit_sparks = *spark_timer >= BURNING_TREE_SPARK_INTERVAL;

    if emit_smoke {
        *smoke_timer -= BURNING_TREE_SMOKE_INTERVAL;
    }
    if emit_sparks {
        *spark_timer -= BURNING_TREE_SPARK_INTERVAL;
    }

    for tree in &burning_trees {
        emit_burning_vfx(
            &mut commands,
            &visual_assets,
            tree.center.x,
            tree.center.z,
            tree.height,
            tree.radius,
            3,
            3,
            t,
            emit_smoke,
            emit_sparks,
        );
    }
}
