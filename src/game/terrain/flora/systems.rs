use bevy::prelude::*;
use rand::prelude::*;

use super::components::Flora;
use super::constants::*;
use super::resources::FloraAssets;
use crate::config::GameConfig;
use crate::config::save_data::SavedFlora;
use crate::game::battlefield::constants::{
    LAVA_POOL_POSITION, LAVA_POOL_RADIUS, WATER_POOL_POSITION, WATER_POOL_RADIUS,
};
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::seeded_rng::resources::{SEED_PURPOSE_FLORA, derive_seed};
use crate::game::shared_systems::{ShadowAssets, spawn_terrain_shadow};
use crate::game::units::components::Corpse;

/// Generates random flora positions and stores them in GameConfig.
///
/// Called once when a wizard's first battle has no saved flora.
/// Uses seeded RNG when a seed is available for deterministic generation.
pub(in crate::game) fn generate_flora_positions(config: &mut GameConfig) {
    let mut rng: Box<dyn RngCore> = match config.seed {
        Some(seed) => Box::new(StdRng::seed_from_u64(derive_seed(
            seed,
            config.current_level,
            SEED_PURPOSE_FLORA,
        ))),
        None => Box::new(rand::thread_rng()),
    };

    let lava_xz = Vec2::new(LAVA_POOL_POSITION.x, LAVA_POOL_POSITION.z);
    let water_xz = Vec2::new(WATER_POOL_POSITION.x, WATER_POOL_POSITION.z);
    // Add margin around hazard pools
    let lava_avoid_sq = (LAVA_POOL_RADIUS + 100.0) * (LAVA_POOL_RADIUS + 100.0);
    let water_avoid_sq = (WATER_POOL_RADIUS + 100.0) * (WATER_POOL_RADIUS + 100.0);

    let mut id = 0u32;
    let mut attempts = 0u32;

    while id < FLORA_COUNT && attempts < FLORA_COUNT * 10 {
        attempts += 1;

        let x = rng.gen_range(FLORA_MIN_X..=FLORA_MAX_X);
        let z = rng.gen_range(FLORA_MIN_Z..=FLORA_MAX_Z);
        let pos = Vec2::new(x, z);

        // Skip positions near lava or water pools
        if pos.distance_squared(lava_xz) < lava_avoid_sq {
            continue;
        }
        if pos.distance_squared(water_xz) < water_avoid_sq {
            continue;
        }

        let sprite_index = rng.gen_range(0..FLORA_SPRITE_COUNT as u8);

        config.saved_flora.push(SavedFlora {
            id,
            x,
            z,
            sprite_index,
        });
        id += 1;
    }
}

/// Spawns a single flora entity from saved data, with a shadow underneath.
pub(in crate::game) fn spawn_single_flora(
    commands: &mut Commands,
    flora_assets: &FloraAssets,
    shadow_assets: &ShadowAssets,
    saved: &SavedFlora,
) {
    let sprite_idx = (saved.sprite_index as usize).min(FLORA_SPRITE_COUNT - 1);
    let spawn_y = FLORA_SPRITE_HEIGHT / 2.0 + 1.0;

    commands.spawn((
        Mesh3d(flora_assets.mesh.clone()),
        MeshMaterial3d(flora_assets.materials[sprite_idx].clone()),
        Transform::from_xyz(saved.x, spawn_y, saved.z),
        Flora { saved_id: saved.id },
        Billboard,
        OnGameplayScreen,
    ));

    spawn_terrain_shadow(
        commands,
        shadow_assets,
        saved.x,
        saved.z,
        FLORA_SHADOW_SCALE,
    );
}

/// Removes flora when living units walk over them.
///
/// Permanently removes trampled flora from both the world and the save data.
pub(super) fn trample_flora(
    mut commands: Commands,
    flora_query: Query<(Entity, &Transform, &Flora)>,
    unit_query: Query<
        &Transform,
        (
            With<crate::game::components::Velocity>,
            Without<Corpse>,
            Without<Flora>,
        ),
    >,
    mut config: ResMut<GameConfig>,
) {
    let mut trampled_ids: Vec<u32> = Vec::new();

    for (flora_entity, flora_transform, flora) in &flora_query {
        let flora_xz = Vec2::new(flora_transform.translation.x, flora_transform.translation.z);

        for unit_transform in &unit_query {
            let unit_xz = Vec2::new(unit_transform.translation.x, unit_transform.translation.z);

            if unit_xz.distance_squared(flora_xz) <= FLORA_TRAMPLE_RADIUS_SQ {
                commands.entity(flora_entity).try_despawn();
                trampled_ids.push(flora.saved_id);
                break;
            }
        }
    }

    if !trampled_ids.is_empty() {
        config.saved_flora.retain(|f| !trampled_ids.contains(&f.id));
    }
}
