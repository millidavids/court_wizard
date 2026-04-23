use bevy::prelude::*;
use rand::prelude::*;

use crate::config::GameConfig;
use crate::config::save_data::{SavedBoulder, SavedBush, SavedPond, SavedTree};
use crate::game::battlefield::constants::{
    LAVA_POOL_POSITION, LAVA_POOL_RADIUS, WATER_POOL_POSITION, WATER_POOL_RADIUS,
};
use crate::game::constants::get_tier;
use crate::game::seeded_rng::resources::{SEED_PURPOSE_TERRAIN, derive_seed};
use crate::game::terrain::boulder::constants::{BOULDER_SPRITE_COUNT, ROCK_RADIUS};
use crate::game::terrain::bush::constants::*;
use crate::game::terrain::pond::constants::*;
use crate::game::terrain::tree::constants::*;

// ===== Terrain placement bounds =====

const TERRAIN_MIN_X: f32 = -2200.0;
const TERRAIN_MAX_X: f32 = 600.0;
const TERRAIN_MIN_Z: f32 = -2200.0;
const TERRAIN_MAX_Z: f32 = 2200.0;

const MIN_TERRAIN_SEPARATION: f32 = 100.0;

const CASTLE_EXCLUSION_RADIUS: f32 = 500.0;
const CASTLE_CENTER_XZ: Vec2 = Vec2::new(-1700.0, 1700.0);

const SPAWN_CORRIDOR_MIN_X: f32 = 500.0;
const SPAWN_CORRIDOR_1_Z: f32 = -375.0;
const SPAWN_CORRIDOR_2_Z: f32 = -1575.0;
const SPAWN_CORRIDOR_RADIUS: f32 = 200.0;

// ===== Size variation =====

/// Min/max scale multiplier for trees, bushes, and boulders.
const SCALE_MIN: f32 = 0.8;
const SCALE_MAX: f32 = 1.2;

// ===== Boulder terrain counts =====

const BOULDER_BASE_COUNT_MIN: u32 = 2;
const BOULDER_BASE_COUNT_MAX: u32 = 6;

/// Generates all terrain (trees, ponds, bushes, boulders) for the first battle.
pub(in crate::game) fn generate_terrain(config: &mut GameConfig, level: u32, terrain_density: f32) {
    if terrain_density <= 0.0 {
        return;
    }

    let seed = config
        .seed
        .map(|s| derive_seed(s, level, SEED_PURPOSE_TERRAIN));
    let mut rng: Box<dyn RngCore> = match seed {
        Some(s) => Box::new(StdRng::seed_from_u64(s)),
        None => Box::new(rand::thread_rng()),
    };

    let tier = get_tier(level);

    // Collect all placed positions for separation checks
    let mut placed: Vec<(Vec2, f32)> = Vec::new();

    // Include existing flora as obstacles
    for flora in &config.saved_flora {
        placed.push((Vec2::new(flora.x, flora.z), 30.0));
    }

    // Pre-compute exclusion zones
    let lava_xz = Vec2::new(LAVA_POOL_POSITION.x, LAVA_POOL_POSITION.z);
    let water_xz = Vec2::new(WATER_POOL_POSITION.x, WATER_POOL_POSITION.z);
    let lava_avoid = LAVA_POOL_RADIUS + 150.0;
    let water_avoid = WATER_POOL_RADIUS + 150.0;
    let spawn_1 = Vec2::new(SPAWN_CORRIDOR_MIN_X, SPAWN_CORRIDOR_1_Z);
    let spawn_2 = Vec2::new(SPAWN_CORRIDOR_MIN_X, SPAWN_CORRIDOR_2_Z);

    let is_valid_position = |pos: Vec2, radius: f32, placed: &[(Vec2, f32)]| -> bool {
        if pos.x - radius < TERRAIN_MIN_X
            || pos.x + radius > TERRAIN_MAX_X
            || pos.y - radius < TERRAIN_MIN_Z
            || pos.y + radius > TERRAIN_MAX_Z
        {
            return false;
        }
        if pos.distance(lava_xz) < lava_avoid + radius {
            return false;
        }
        if pos.distance(water_xz) < water_avoid + radius {
            return false;
        }
        if pos.distance(CASTLE_CENTER_XZ) < CASTLE_EXCLUSION_RADIUS + radius {
            return false;
        }
        if pos.distance(spawn_1) < SPAWN_CORRIDOR_RADIUS + radius {
            return false;
        }
        if pos.distance(spawn_2) < SPAWN_CORRIDOR_RADIUS + radius {
            return false;
        }
        for &(other_pos, other_radius) in placed {
            if pos.distance(other_pos) < radius + other_radius + MIN_TERRAIN_SEPARATION {
                return false;
            }
        }
        true
    };

    let scale_count = |min: u32, max: u32| -> u32 {
        let base = min + ((max - min) as f32 * (tier as f32 / 5.0).min(1.0)) as u32;
        (base as f32 * terrain_density).round() as u32
    };

    let random_scale = |rng: &mut dyn RngCore| -> f32 { rng.gen_range(SCALE_MIN..=SCALE_MAX) };

    // Generate boulders
    let boulder_count = scale_count(BOULDER_BASE_COUNT_MIN, BOULDER_BASE_COUNT_MAX);
    for _ in 0..boulder_count {
        let scale = random_scale(&mut *rng);
        let radius = ROCK_RADIUS * scale;
        if let Some(pos) = try_place(
            &mut *rng,
            radius,
            &placed,
            &is_valid_position,
            TERRAIN_MIN_X,
            TERRAIN_MAX_X,
            TERRAIN_MIN_Z,
            TERRAIN_MAX_Z,
        ) {
            let sprite_index = rng.gen_range(0..BOULDER_SPRITE_COUNT as u8);
            config.saved_boulders.push(SavedBoulder {
                x: pos.x,
                z: pos.y,
                scale,
                sprite_index,
            });
            placed.push((pos, radius));
        }
    }

    // Generate forest along the left wall, starting 100 units from the battlefield edge
    // (Z = -2900) and thinning toward the center of the battlefield.
    {
        use crate::game::terrain::tree::constants::tree_radius_for_variant;

        const FOREST_Z_START: f32 = -2950.0; // 50 units from left wall at Z = -3000
        const FOREST_Z_END: f32 = -200.0;
        const FOREST_BANDS: u32 = 6;
        let trees_per_band: [u32; 6] = [15, 12, 8, 4, 2, 1];
        let band_depth = (FOREST_Z_END - FOREST_Z_START) / FOREST_BANDS as f32;

        // Forest-specific validation — allows Z down to FOREST_Z_START
        // and uses tighter separation (50 units) for a dense canopy.
        const FOREST_MIN_X: f32 = -3000.0;
        let is_valid_forest = |pos: Vec2, radius: f32, placed: &[(Vec2, f32)]| -> bool {
            if pos.x - radius < FOREST_MIN_X
                || pos.x + radius > TERRAIN_MAX_X
                || pos.y - radius < FOREST_Z_START
                || pos.y + radius > TERRAIN_MAX_Z
            {
                return false;
            }
            if pos.distance(lava_xz) < lava_avoid + radius {
                return false;
            }
            if pos.distance(water_xz) < water_avoid + radius {
                return false;
            }
            if pos.distance(CASTLE_CENTER_XZ) < CASTLE_EXCLUSION_RADIUS + radius {
                return false;
            }
            if pos.distance(spawn_1) < SPAWN_CORRIDOR_RADIUS + radius {
                return false;
            }
            if pos.distance(spawn_2) < SPAWN_CORRIDOR_RADIUS + radius {
                return false;
            }
            for &(other_pos, other_radius) in placed {
                if pos.distance(other_pos) < radius + other_radius + 50.0 {
                    return false;
                }
            }
            true
        };

        // No-collision validation for flora/bushes placed BETWEEN trees.
        // Only checks boundary — flora can overlap trees and other terrain.
        let is_valid_forest_flora = |pos: Vec2, _radius: f32, _placed: &[(Vec2, f32)]| -> bool {
            pos.x >= FOREST_MIN_X
                && pos.x <= TERRAIN_MAX_X
                && pos.y >= FOREST_Z_START
                && pos.y <= TERRAIN_MAX_Z
        };

        for band in 0..FOREST_BANDS {
            let band_z_min = FOREST_Z_START + band as f32 * band_depth;
            let band_z_max = band_z_min + band_depth;
            let target = (trees_per_band[band as usize] as f32 * terrain_density).round() as u32;

            for _ in 0..target {
                let scale = random_scale(&mut *rng);
                let sprite_index = rng.gen_range(0..TREE_SPRITE_COUNT as u8);
                let radius = tree_radius_for_variant(sprite_index) * scale;
                if let Some(pos) = try_place(
                    &mut *rng,
                    radius,
                    &placed,
                    &is_valid_forest,
                    FOREST_MIN_X,
                    TERRAIN_MAX_X,
                    band_z_min,
                    band_z_max,
                ) {
                    config.saved_trees.push(SavedTree {
                        x: pos.x,
                        z: pos.y,
                        scale,
                        sprite_index,
                    });
                    placed.push((pos, radius));
                }
            }

            // Scatter bushes and boulders between the trees in this band.
            // These use the flora validation (no collision checks) so they
            // can overlap with trees for a dense undergrowth look.
            let bush_target = target / 2;
            for _ in 0..bush_target {
                let scale = random_scale(&mut *rng);
                let sprite_index = rng.gen_range(0..BUSH_SPRITE_COUNT as u8);
                if let Some(pos) = try_place(
                    &mut *rng,
                    0.0,
                    &placed,
                    &is_valid_forest_flora,
                    FOREST_MIN_X,
                    TERRAIN_MAX_X,
                    band_z_min,
                    band_z_max,
                ) {
                    config.saved_bushes.push(SavedBush {
                        x: pos.x,
                        z: pos.y,
                        scale,
                        sprite_index,
                    });
                }
            }

            let boulder_target = target / 4;
            for _ in 0..boulder_target {
                let scale = random_scale(&mut *rng) * 0.8; // slightly smaller in forest
                let sprite_index = rng.gen_range(0..BOULDER_SPRITE_COUNT as u8);
                if let Some(pos) = try_place(
                    &mut *rng,
                    0.0,
                    &placed,
                    &is_valid_forest_flora,
                    FOREST_MIN_X,
                    TERRAIN_MAX_X,
                    band_z_min,
                    band_z_max,
                ) {
                    config.saved_boulders.push(SavedBoulder {
                        x: pos.x,
                        z: pos.y,
                        scale,
                        sprite_index,
                    });
                    // Boulders DO block pathfinding, so add to placed
                    let radius = ROCK_RADIUS * scale;
                    placed.push((pos, radius));
                }
            }
        }
    }

    // Generate scattered trees across the battlefield
    let tree_count = scale_count(TREE_BASE_COUNT_MIN, TREE_BASE_COUNT_MAX);
    for _ in 0..tree_count {
        let scale = random_scale(&mut *rng);
        let sprite_index = rng.gen_range(0..TREE_SPRITE_COUNT as u8);
        let radius = tree_radius_for_variant(sprite_index) * scale;
        if let Some(pos) = try_place(
            &mut *rng,
            radius,
            &placed,
            &is_valid_position,
            TERRAIN_MIN_X,
            TERRAIN_MAX_X,
            TERRAIN_MIN_Z,
            TERRAIN_MAX_Z,
        ) {
            config.saved_trees.push(SavedTree {
                x: pos.x,
                z: pos.y,
                scale,
                sprite_index,
            });
            placed.push((pos, radius));
        }
    }

    // Generate ponds (radius already varies, no extra scale needed)
    let pond_count = scale_count(POND_BASE_COUNT_MIN, POND_BASE_COUNT_MAX);
    for _ in 0..pond_count {
        let radius = rng.gen_range(POND_RADIUS_MIN..=POND_RADIUS_MAX);
        if let Some(pos) = try_place(
            &mut *rng,
            radius,
            &placed,
            &is_valid_position,
            TERRAIN_MIN_X,
            TERRAIN_MAX_X,
            TERRAIN_MIN_Z,
            TERRAIN_MAX_Z,
        ) {
            config.saved_ponds.push(SavedPond {
                x: pos.x,
                z: pos.y,
                radius,
            });
            placed.push((pos, radius));
        }
    }

    // Generate bushes
    let bush_count = scale_count(BUSH_BASE_COUNT_MIN, BUSH_BASE_COUNT_MAX);
    for _ in 0..bush_count {
        let scale = random_scale(&mut *rng);
        let radius = BUSH_RADIUS * scale;
        if let Some(pos) = try_place(
            &mut *rng,
            radius,
            &placed,
            &is_valid_position,
            TERRAIN_MIN_X,
            TERRAIN_MAX_X,
            TERRAIN_MIN_Z,
            TERRAIN_MAX_Z,
        ) {
            let sprite_index = rng.gen_range(0..BUSH_SPRITE_COUNT as u8);
            config.saved_bushes.push(SavedBush {
                x: pos.x,
                z: pos.y,
                scale,
                sprite_index,
            });
            placed.push((pos, radius));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn try_place(
    rng: &mut dyn RngCore,
    radius: f32,
    placed: &[(Vec2, f32)],
    is_valid: &dyn Fn(Vec2, f32, &[(Vec2, f32)]) -> bool,
    x_min: f32,
    x_max: f32,
    z_min: f32,
    z_max: f32,
) -> Option<Vec2> {
    for _ in 0..80 {
        let x = rng.gen_range(x_min..=x_max);
        let z = rng.gen_range(z_min..=z_max);
        let pos = Vec2::new(x, z);
        if is_valid(pos, radius, placed) {
            return Some(pos);
        }
    }
    None
}
