use super::super::components::{WallHealth, WallOfStone, WallRising};
use super::super::constants::*;
use super::super::wall_material::WallOfStoneMaterial;
use crate::game::battlefield::trampling::constants::TRAMPLING_CELL_SIZE;
use crate::game::battlefield::trampling::resources::TramplingGrid;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

/// Tints wall material toward the damaged color based on remaining HP.
///
/// On first damage, clones the shared material into a per-wall instance so
/// tinting one wall doesn't affect others. Uses the `damage_tint` uniform
/// which the shader applies as a final lerp over the computed texture/noise.
pub fn update_wall_damage_tint(
    mut walls: Query<(&WallHealth, &mut MeshMaterial3d<WallOfStoneMaterial>), With<WallOfStone>>,
    mut materials: ResMut<Assets<WallOfStoneMaterial>>,
    visual_assets: Res<SpellVisualAssets>,
) {
    let damaged = WALL_DAMAGED_COLOR.to_srgba();

    for (wall_health, mut material_handle) in &mut walls {
        if wall_health.current >= wall_health.max {
            continue;
        }

        // If still using the shared material, clone it into a per-wall instance
        if material_handle.0 == visual_assets.wall_of_stone {
            let Some(shared_mat) = materials.get(&visual_assets.wall_of_stone) else {
                continue;
            };
            let cloned = shared_mat.clone();
            material_handle.0 = materials.add(cloned);
        }

        let Some(mut material) = materials.get_mut(&material_handle.0) else {
            continue;
        };

        // damage_tint.a goes from 0 (full HP) to 1 (0 HP)
        let hp_frac = wall_health.fraction();
        material.damage_tint = Vec4::new(damaged.red, damaged.green, damaged.blue, 1.0 - hp_frac);
    }
}

/// Animates walls rising up from the ground when first placed.
/// Moves the wall from below ground to its final position over WALL_RISE_DURATION.
pub fn animate_rising_walls(
    mut commands: Commands,
    time: Res<Time>,
    mut walls: Query<(Entity, &WallOfStone, &mut WallRising, &mut Transform)>,
) {
    let delta = time.delta_secs();
    for (entity, wall, mut rising, mut transform) in &mut walls {
        rising.elapsed += delta;
        let progress = rising.progress();

        // Ease-out: starts fast, slows at the top
        let eased = 1.0 - (1.0 - progress) * (1.0 - progress);

        // Move wall from underground to final height
        let final_y = wall.height / 2.0;
        transform.translation.y = final_y * eased - wall.height * (1.0 - eased);

        if progress >= 1.0 {
            // Snap to final position and remove rising component
            transform.translation.y = final_y;
            commands.entity(entity).remove::<WallRising>();
        }
    }
}

/// Applies trampling around a wall when it finishes rising.
/// Creates a dirt patch around the wall footprint as if the ground was churned up.
pub fn apply_wall_trampling(
    walls: Query<(&WallOfStone, &WallRising)>,
    mut grid: Option<ResMut<TramplingGrid>>,
    time: Res<Time>,
) {
    let Some(ref mut grid) = grid else {
        return;
    };
    let delta = time.delta_secs();
    let cell_size = TRAMPLING_CELL_SIZE;

    for (wall, rising) in &walls {
        // Only apply once as the wall nears the end of its rise
        let prev_progress = ((rising.elapsed - delta) / rising.duration).clamp(0.0, 1.0);
        if prev_progress >= 0.5 || rising.progress() < 0.5 {
            continue;
        }

        // Compute AABB of the wall footprint with a buffer for the disturbed area
        let buffer = 30.0;
        let bounds = wall.obstacle_bounds();
        let min_x = bounds[0] - buffer;
        let min_z = bounds[1] - buffer;
        let max_x = bounds[2] + buffer;
        let max_z = bounds[3] + buffer;

        // Iterate over grid cells in the AABB
        let mut x = min_x;
        while x <= max_x {
            let mut z = min_z;
            while z <= max_z {
                let dist = wall.distance_to_surface(Vec3::new(x, 0.0, z));
                // Strong trampling on the wall footprint, fading outward
                let intensity = if dist < 1.0 {
                    0.5
                } else {
                    (1.0 - (dist / buffer).min(1.0)) * 0.4
                };
                if intensity > 0.0
                    && let Some(idx) = grid.world_to_index(x, z)
                {
                    grid.values[idx] = (grid.values[idx] + intensity).min(1.0);
                }
                z += cell_size;
            }
            x += cell_size;
        }
        grid.dirty = true;
    }
}

/// Spawns dust puffs along walls that are rising or sinking.
pub fn spawn_wall_dust(
    mut commands: Commands,
    rising_walls: Query<(&WallOfStone, &WallRising)>,
    sinking_walls: Query<&WallOfStone, Without<WallRising>>,
    visual_assets: Res<crate::game::units::wizard::spells::visual_assets::SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
    mut pending_cast_events: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    *timer += time.delta_secs();
    if *timer < WALL_DUST_INTERVAL {
        return;
    }
    *timer -= WALL_DUST_INTERVAL;

    let t = time.elapsed_secs();

    // Spawn dust for rising walls
    for (wall, _rising) in &rising_walls {
        spawn_dust_along_wall(
            &mut commands,
            &visual_assets,
            wall,
            t,
            &mut pending_cast_events,
        );
    }

    // Spawn dust for sinking walls
    for wall in &sinking_walls {
        if wall.sinking {
            spawn_dust_along_wall(
                &mut commands,
                &visual_assets,
                wall,
                t,
                &mut pending_cast_events,
            );
        }
    }
}

/// Helper: spawns dust puffs distributed along a wall's length.
fn spawn_dust_along_wall(
    commands: &mut Commands,
    assets: &crate::game::units::wizard::spells::visual_assets::SpellVisualAssets,
    wall: &WallOfStone,
    time_secs: f32,
    pending: &mut crate::game::multiplayer::spell_sync::PendingCastEvents,
) {
    let wall_len = wall.half_length * 2.0;
    let num_points = ((wall_len / 50.0) as usize).max(2);

    for j in 0..num_points {
        let frac = (j as f32 + (time_secs * 2.3 + j as f32 * 1.7).fract()) / num_points as f32;
        let pos = wall.center - wall.forward * wall.half_length
            + wall.forward * (wall_len * frac.clamp(0.0, 1.0));

        crate::game::units::wizard::spells::vfx::systems::spawn_dust_smoke_synced(
            commands,
            assets,
            pending,
            pos,
            wall.half_width,
            WALL_DUST_PUFFS_PER_POINT,
            time_secs + j as f32,
        );
    }
}
