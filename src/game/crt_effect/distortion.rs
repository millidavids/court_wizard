//! Distortion position updates for lensing, heat, and teleport effects.

use bevy::prelude::*;

use super::components::{HeatDistortionSettings, LensingSettings, TeleportDistortionSettings};
use super::constants::{LENSING_INFLUENCE_MULT, LENSING_STRENGTH};
use super::systems::ndc_to_uv;
use crate::game::battlefield::components::LavaPool;
use crate::game::terrain::bush::components::{BurningBush, Bush};
use crate::game::terrain::tree::components::{BurningTree, Tree};
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;

pub(super) fn update_lensing_positions(
    rifts: Query<&crate::game::units::wizard::spells::teleport::components::DimensionalRift>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut lensing_query: Query<&mut LensingSettings>,
) {
    let Ok(mut settings) = lensing_query.single_mut() else {
        return;
    };

    // Reset lensing data
    settings.lensing_count = 0.0;
    settings.lensing_strength = LENSING_STRENGTH;
    settings.lensing_darkening = 0.0;
    settings.lensing_0_x = 0.0;
    settings.lensing_0_y = 0.0;
    settings.lensing_0_radius = 0.0;
    settings.lensing_1_x = 0.0;
    settings.lensing_1_y = 0.0;
    settings.lensing_1_radius = 0.0;
    settings.lensing_2_x = 0.0;
    settings.lensing_2_y = 0.0;
    settings.lensing_2_radius = 0.0;
    settings.lensing_3_x = 0.0;
    settings.lensing_3_y = 0.0;
    settings.lensing_3_radius = 0.0;

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // Slots 0-1: previously reserved for black hole lensing — now disabled.
    // Black holes render as plain opaque black spheres without any
    // screen-space distortion, matching the simpler single-player look.
    // The slots are left zeroed so the shader's branchless `step()` checks
    // ignore them.
    let count = 0u32;

    // Slots 2-3: Dimensional Rift endpoints (lensing only, no darkening)
    let rift_radius =
        crate::game::units::wizard::spells::teleport::vfx_constants::RIFT_LENSING_RADIUS;
    let mut rift_slot = 2u32;
    'rift_loop: for rift in &rifts {
        for &pos in &[rift.source_pos, rift.dest_pos] {
            if rift_slot > 3 {
                break 'rift_loop;
            }

            let Some(ndc) = camera.world_to_ndc(camera_transform, pos) else {
                continue;
            };
            let uv = ndc_to_uv(ndc);

            if uv.x < -0.3 || uv.x > 1.3 || uv.y < -0.3 || uv.y > 1.3 {
                continue;
            }

            let edge_point = pos + camera_transform.right() * rift_radius;
            let Some(edge_ndc) = camera.world_to_ndc(camera_transform, edge_point) else {
                continue;
            };
            let edge_uv = ndc_to_uv(edge_ndc);
            let screen_radius = (edge_uv.x - uv.x).abs() * LENSING_INFLUENCE_MULT;

            if screen_radius < 0.001 {
                continue;
            }

            match rift_slot {
                2 => {
                    settings.lensing_2_x = uv.x;
                    settings.lensing_2_y = uv.y;
                    settings.lensing_2_radius = screen_radius;
                }
                3 => {
                    settings.lensing_3_x = uv.x;
                    settings.lensing_3_y = uv.y;
                    settings.lensing_3_radius = screen_radius;
                }
                _ => {}
            }
            rift_slot += 1;
        }
    }

    // lensing_count must reflect the highest occupied slot + 1, not total sources.
    // Rift endpoints live in slots 2-3, so even with 0 black holes we need count >= 3/4
    // for the shader's branchless step() checks to activate those slots.
    let max_slot = if rift_slot > 2 { rift_slot } else { count };
    settings.lensing_count = max_slot as f32;
}

/// Projects active wall of fire positions to viewport-local UV space for heat distortion.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_heat_distortion_positions(
    walls: Query<&WallOfFireEffect>,
    explosions: Query<&FireballExplosion>,
    burning_trees: Query<&Tree, With<BurningTree>>,
    burning_bushes: Query<&Bush, With<BurningBush>>,
    lava_pools: Query<&Transform, With<LavaPool>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut distortion_query: Query<&mut HeatDistortionSettings>,
    time: Res<Time>,
) {
    let Ok(mut settings) = distortion_query.single_mut() else {
        return;
    };

    // Reset
    settings.count = 0.0;
    settings.time = time.elapsed_secs();

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let mut count = 0u32;

    // Walls of fire get priority (most dramatic linear distortion)
    for wall in &walls {
        if count >= 4 {
            break;
        }

        let Some(start_ndc) = camera.world_to_ndc(camera_transform, wall.start) else {
            continue;
        };
        let Some(end_ndc) = camera.world_to_ndc(camera_transform, wall.end) else {
            continue;
        };

        let start_uv = ndc_to_uv(start_ndc);
        let end_uv = ndc_to_uv(end_ndc);

        if (start_uv.x < -0.3 && end_uv.x < -0.3)
            || (start_uv.x > 1.3 && end_uv.x > 1.3)
            || (start_uv.y < -0.3 && end_uv.y < -0.3)
            || (start_uv.y > 1.3 && end_uv.y > 1.3)
        {
            continue;
        }

        let mid = (wall.start + wall.end) / 2.0;
        let edge = mid + camera_transform.right() * wall.half_width * 3.0;
        let Some(mid_ndc) = camera.world_to_ndc(camera_transform, mid) else {
            continue;
        };
        let Some(edge_ndc) = camera.world_to_ndc(camera_transform, edge) else {
            continue;
        };
        let radius = (ndc_to_uv(edge_ndc).x - ndc_to_uv(mid_ndc).x)
            .abs()
            .max(0.02);

        set_distortion_slot(&mut settings, count, start_uv, end_uv, radius);
        count += 1;
    }

    // Fill remaining slots with point fire sources (start == end for points)
    // Fireball explosions
    for explosion in &explosions {
        if count >= 4 {
            break;
        }
        if let Some(uv_radius) = project_point_source(
            camera,
            camera_transform,
            explosion.origin,
            explosion.max_radius,
        ) {
            set_distortion_slot(&mut settings, count, uv_radius.0, uv_radius.0, uv_radius.1);
            count += 1;
        }
    }

    // Burning trees
    for tree in &burning_trees {
        if count >= 4 {
            break;
        }
        if let Some(uv_radius) =
            project_point_source(camera, camera_transform, tree.center, tree.radius * 2.0)
        {
            set_distortion_slot(&mut settings, count, uv_radius.0, uv_radius.0, uv_radius.1);
            count += 1;
        }
    }

    // Burning bushes
    for bush in &burning_bushes {
        if count >= 4 {
            break;
        }
        if let Some(uv_radius) =
            project_point_source(camera, camera_transform, bush.center, bush.radius * 2.0)
        {
            set_distortion_slot(&mut settings, count, uv_radius.0, uv_radius.0, uv_radius.1);
            count += 1;
        }
    }

    // Lava pools
    for transform in &lava_pools {
        if count >= 4 {
            break;
        }
        if let Some(uv_radius) =
            project_point_source(camera, camera_transform, transform.translation, 60.0)
        {
            set_distortion_slot(&mut settings, count, uv_radius.0, uv_radius.0, uv_radius.1);
            count += 1;
        }
    }

    settings.count = count as f32;
}

/// Projects a world-space point source to screen UV and computes its influence radius.
/// Returns `None` if the point is off-screen.
fn project_point_source(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    world_pos: Vec3,
    world_radius: f32,
) -> Option<(Vec2, f32)> {
    let ndc = camera.world_to_ndc(camera_transform, world_pos)?;
    let uv = ndc_to_uv(ndc);
    if uv.x < -0.3 || uv.x > 1.3 || uv.y < -0.3 || uv.y > 1.3 {
        return None;
    }
    let edge = world_pos + camera_transform.right() * world_radius;
    let edge_ndc = camera.world_to_ndc(camera_transform, edge)?;
    let radius = (ndc_to_uv(edge_ndc).x - uv.x).abs().max(0.02);
    Some((uv, radius))
}

/// Sets one of the 4 distortion slots by index.
fn set_distortion_slot(
    settings: &mut HeatDistortionSettings,
    index: u32,
    start_uv: Vec2,
    end_uv: Vec2,
    radius: f32,
) {
    match index {
        0 => {
            settings.wall_0_start_x = start_uv.x;
            settings.wall_0_start_y = start_uv.y;
            settings.wall_0_end_x = end_uv.x;
            settings.wall_0_end_y = end_uv.y;
            settings.wall_0_radius = radius;
        }
        1 => {
            settings.wall_1_start_x = start_uv.x;
            settings.wall_1_start_y = start_uv.y;
            settings.wall_1_end_x = end_uv.x;
            settings.wall_1_end_y = end_uv.y;
            settings.wall_1_radius = radius;
        }
        2 => {
            settings.wall_2_start_x = start_uv.x;
            settings.wall_2_start_y = start_uv.y;
            settings.wall_2_end_x = end_uv.x;
            settings.wall_2_end_y = end_uv.y;
            settings.wall_2_radius = radius;
        }
        3 => {
            settings.wall_3_start_x = start_uv.x;
            settings.wall_3_start_y = start_uv.y;
            settings.wall_3_end_x = end_uv.x;
            settings.wall_3_end_y = end_uv.y;
            settings.wall_3_radius = radius;
        }
        _ => {}
    }
}

/// Projects active teleport warp effect positions to viewport-local UV for the distortion shader.
pub(super) fn update_teleport_distortion_positions(
    warps: Query<&crate::game::units::wizard::spells::teleport::vfx_components::TeleportWarpEffect>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut distortion_query: Query<&mut TeleportDistortionSettings>,
    time: Res<Time>,
) {
    let Ok(mut settings) = distortion_query.single_mut() else {
        return;
    };

    // Reset
    settings.count = 0.0;
    settings.time = time.elapsed_secs();

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let influence_mult =
        crate::game::units::wizard::spells::teleport::vfx_constants::RIPPLE_INFLUENCE_MULT;

    let mut count = 0u32;
    let mut has_persistent = false;
    for warp in &warps {
        if count >= 4 {
            break;
        }

        if warp.duration == 0.0 {
            has_persistent = true;
        }

        // Project center to screen UV
        let Some(ndc) = camera.world_to_ndc(camera_transform, warp.position) else {
            continue;
        };
        let uv = ndc_to_uv(ndc);

        // Skip if too far off screen
        if uv.x < -0.5 || uv.x > 1.5 || uv.y < -0.5 || uv.y > 1.5 {
            continue;
        }

        // Project a point at the edge to get screen-space radius
        let edge_point = warp.position + camera_transform.right() * warp.radius;
        let Some(edge_ndc) = camera.world_to_ndc(camera_transform, edge_point) else {
            continue;
        };
        let edge_uv = ndc_to_uv(edge_ndc);
        let screen_radius = (edge_uv.x - uv.x).abs() * influence_mult;

        if screen_radius < 0.001 {
            continue;
        }

        settings.set_point(count, uv.x, uv.y, screen_radius, warp.intensity);
        count += 1;
    }
    settings.count = count as f32;

    // Use rift strength if any persistent warp effects are active, otherwise one-shot strength
    if has_persistent {
        settings.strength =
            crate::game::units::wizard::spells::teleport::vfx_constants::RIFT_RIPPLE_STRENGTH;
    } else {
        settings.strength =
            crate::game::units::wizard::spells::teleport::vfx_constants::RIPPLE_STRENGTH;
    }
}
