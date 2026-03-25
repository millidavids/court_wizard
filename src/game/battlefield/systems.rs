use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use rand::Rng;

use super::components::{
    Battlefield, BattlefieldAssets, Castle, LavaPool, LeftWall, RightWall, WallFloor, WaterRipple,
    WaterRippleAssets,
};
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::*;

/// Sets up the battlefield and castle when entering the InGame state.
///
/// Spawns the battlefield ground plane, castle wall image, and point light in 3D space.
pub fn setup_battlefield(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    battlefield_assets: &BattlefieldAssets,
) {
    // Add a light source so we can see 3D objects
    commands.spawn((
        PointLight {
            intensity: 2_000_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 1000.0, 0.0),
        OnGameplayScreen,
    ));

    // Spawn battlefield as a grid of textured ground tiles
    spawn_ground_tiles(commands, meshes, materials, battlefield_assets);

    // Spawn castle wall as a textured plane the wizard stands on
    spawn_castle_wall(
        commands,
        meshes,
        materials,
        battlefield_assets,
        CASTLE_POSITION,
        CASTLE_ROTATION_DEGREES,
        OnGameplayScreen,
    );

    // Spawn wall backdrops (vertical walls get depth_bias so they render behind game entities)
    // Right wall uses Mask alpha so it occludes units spawning behind it;
    // tunnel areas in the art are transparent and become real holes.
    spawn_right_wall_backdrop(
        commands,
        meshes,
        materials,
        battlefield_assets.right_wall.clone(),
        RIGHT_WALL_WIDTH,
        RIGHT_WALL_HEIGHT,
        Transform::from_translation(RIGHT_WALL_POSITION).with_rotation(Quat::from_rotation_y(
            RIGHT_WALL_ROTATION_DEGREES.to_radians(),
        )),
    );
    spawn_wall_backdrop(
        commands,
        meshes,
        materials,
        battlefield_assets.left_wall.clone(),
        LEFT_WALL_WIDTH,
        LEFT_WALL_HEIGHT,
        Transform::from_translation(LEFT_WALL_POSITION),
        LeftWall,
        -100.0,
    );

    // Spawn floor between the right and left walls (laid flat, facing up; negative depth bias so effects render on top)
    spawn_wall_backdrop(
        commands,
        meshes,
        materials,
        battlefield_assets.wall_floor.clone(),
        WALL_FLOOR_DEPTH,
        WALL_FLOOR_LENGTH,
        Transform::from_translation(WALL_FLOOR_POSITION)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        WallFloor,
        -1000.0,
    );

    // Spawn lava pool marker (drives fire/smoke/spark effects)
    commands.spawn((
        Transform::from_translation(LAVA_POOL_POSITION),
        Visibility::default(),
        LavaPool,
        OnGameplayScreen,
    ));

    // Pre-allocate annulus mesh for water ripples (inner_radius < outer_radius, thin ring)
    let ripple_mesh = meshes.add(Annulus::new(0.9, 1.0));
    commands.insert_resource(WaterRippleAssets { mesh: ripple_mesh });
}

/// Spawns a textured wall backdrop as a vertical rectangle.
#[allow(clippy::too_many_arguments)]
fn spawn_wall_backdrop<M: Component>(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    texture: Handle<Image>,
    width: f32,
    height: f32,
    transform: Transform,
    marker: M,
    depth_bias: f32,
) {
    let mesh = Rectangle::new(width, height);
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        depth_bias,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        transform,
        marker,
        OnGameplayScreen,
    ));
}

/// Spawns the right wall with AlphaMode::Mask so it writes to the depth buffer,
/// hiding units that spawn behind it. Tunnel areas in the art are transparent.
fn spawn_right_wall_backdrop(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    texture: Handle<Image>,
    width: f32,
    height: f32,
    transform: Transform,
) {
    let mesh = Rectangle::new(width, height);
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Mask(0.5),
        unlit: true,
        cull_mode: None,
        depth_bias: 0.0,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        transform,
        RightWall,
        OnGameplayScreen,
    ));
}

/// Spawns the castle wall as a textured plane at the given position.
///
/// The plane uses the castle_wall.png image at its natural aspect ratio,
/// scaled to CASTLE_WIDTH wide. The wizard and cauldron stand on this plane.
pub fn spawn_castle_wall<M: Component + Clone>(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    battlefield_assets: &BattlefieldAssets,
    castle_position: Vec3,
    rotation_degrees: f32,
    screen_marker: M,
) {
    // Size the plane to match the image's aspect ratio (629x1024), scaled 3x.
    // Width = CASTLE_WIDTH * 3, depth computed from aspect ratio.
    const IMAGE_WIDTH: f32 = 629.0;
    const IMAGE_HEIGHT: f32 = 1024.0;
    let plane_width = CASTLE_WIDTH * 3.0;
    let plane_depth = plane_width * (IMAGE_HEIGHT / IMAGE_WIDTH);

    let wall_mesh = Plane3d::default().mesh().size(plane_width, plane_depth);

    let wall_material = materials.add(StandardMaterial {
        base_color_texture: Some(battlefield_assets.castle_wall.clone()),
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(wall_mesh)),
        MeshMaterial3d(wall_material),
        Transform::from_translation(castle_position)
            .with_rotation(Quat::from_rotation_y(rotation_degrees.to_radians())),
        Castle,
        screen_marker,
    ));
}

// ===== Ground Tile Spawning =====

/// Creates a plane mesh with UVs that sample a specific tile from the sprite sheet.
fn create_tile_mesh(tile_size: f32, tile_index: usize, total_tiles: usize) -> Mesh {
    let half = tile_size / 2.0;
    let u_min = tile_index as f32 / total_tiles as f32;
    let u_max = (tile_index + 1) as f32 / total_tiles as f32;

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-half, 0.0, -half],
            [half, 0.0, -half],
            [half, 0.0, half],
            [-half, 0.0, half],
        ],
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![
            [u_max, 0.0],
            [u_min, 0.0],
            [u_min, 1.0],
            [u_max, 1.0],
        ],
    );
    mesh.insert_indices(Indices::U32(vec![0, 2, 1, 0, 3, 2]));
    mesh
}

/// Picks a weighted-random tile index. Base tiles are heavily favored over flavor tiles.
fn pick_weighted_tile(rng: &mut impl Rng) -> usize {
    let roll: u32 = rng.gen_range(0..TILE_TOTAL_WEIGHT);
    let base_total = TILE_BASE_COUNT as u32 * TILE_BASE_WEIGHT;
    if roll < base_total {
        (roll / TILE_BASE_WEIGHT) as usize
    } else {
        TILE_BASE_COUNT + ((roll - base_total) / TILE_FLAVOR_WEIGHT) as usize
    }
}

/// Spawns a grid of ground tiles covering the battlefield.
fn spawn_ground_tiles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    battlefield_assets: &BattlefieldAssets,
) {
    let half = BATTLEFIELD_SIZE / 2.0;
    let tiles_per_side = (BATTLEFIELD_SIZE / TILE_WORLD_SIZE) as usize;

    // Pre-create one mesh handle per tile variant (shared across all tiles of that type)
    let tile_meshes: Vec<Handle<Mesh>> = (0..TILE_COUNT)
        .map(|i| meshes.add(create_tile_mesh(TILE_WORLD_SIZE, i, TILE_COUNT)))
        .collect();

    // One shared material for all tiles
    let tile_material = materials.add(StandardMaterial {
        base_color_texture: Some(battlefield_assets.battlefield_tiles.clone()),
        base_color: Color::WHITE,
        unlit: true,
        ..default()
    });

    let mut rng = rand::thread_rng();

    for row in 0..tiles_per_side {
        for col in 0..tiles_per_side {
            let x = -half + TILE_WORLD_SIZE * (col as f32 + 0.5);
            let z = -half + TILE_WORLD_SIZE * (row as f32 + 0.5);
            let tile_index = pick_weighted_tile(&mut rng);

            commands.spawn((
                Mesh3d(tile_meshes[tile_index].clone()),
                MeshMaterial3d(tile_material.clone()),
                Transform::from_xyz(x, 0.0, z),
                Battlefield,
                OnGameplayScreen,
            ));
        }
    }
}

// ===== Environmental Effects =====

use crate::game::units::components::{Corpse, Health, RoughTerrainModifier};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Emits fire smoke puffs from the lava pool on the wall floor.
pub fn emit_lava_fire_smoke(
    mut commands: Commands,
    lava_pools: Query<&Transform, With<LavaPool>>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < LAVA_SMOKE_INTERVAL {
        return;
    }
    *timer -= LAVA_SMOKE_INTERVAL;

    let t = time.elapsed_secs();

    for transform in lava_pools.iter() {
        vfx::systems::spawn_fire_orange_smoke(
            &mut commands,
            &visual_assets,
            transform.translation,
            LAVA_POOL_RADIUS,
            4,
            t,
        );
    }
}

/// Emits occasional spark bursts from the lava pool.
pub fn emit_lava_sparks(
    mut commands: Commands,
    lava_pools: Query<&Transform, With<LavaPool>>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < LAVA_SPARK_INTERVAL {
        return;
    }
    *timer -= LAVA_SPARK_INTERVAL;

    let t = time.elapsed_secs();

    for transform in lava_pools.iter() {
        vfx::systems::spawn_fire_sparks(&mut commands, &visual_assets, transform.translation, 6, t);
    }
}

/// Spawns growing, fading annulus ripples on the water pool.
pub fn emit_water_ripples(
    mut commands: Commands,
    ripple_assets: Res<WaterRippleAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    let assets = ripple_assets;

    *timer += time.delta_secs();
    if *timer < WATER_RIPPLE_INTERVAL {
        return;
    }
    *timer -= WATER_RIPPLE_INTERVAL;

    let t = time.elapsed_secs();

    // Spawn 1 ripple per interval at a random position within the pool
    let seed = t * 3.7;
    let angle = seed * 2.39;
    let dist_frac = (seed * 17.3).sin() * 0.5 + 0.5;
    let x = WATER_POOL_POSITION.x + angle.cos() * WATER_POOL_RADIUS * dist_frac * 0.5;
    let z = WATER_POOL_POSITION.z + angle.sin() * WATER_POOL_RADIUS * dist_frac * 0.5;

    let max_scale = WATER_RIPPLE_MAX_SCALE * (0.6 + 0.4 * ((seed * 41.7).sin() * 0.5 + 0.5));
    let lifetime = WATER_RIPPLE_LIFETIME * (0.8 + 0.4 * ((seed * 23.1).sin() * 0.5 + 0.5));

    // Each ripple gets its own material so we can fade alpha independently
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.9, 0.95, 1.0, WATER_RIPPLE_ALPHA),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        Mesh3d(assets.mesh.clone()),
        MeshMaterial3d(mat),
        Transform::from_xyz(x, WATER_POOL_POSITION.y + 2.0, z)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(10.0)),
        WaterRipple {
            lifetime: 0.0,
            max_lifetime: lifetime,
            max_scale,
        },
        OnGameplayScreen,
    ));
}

/// Grows and fades water ripples over their lifetime.
pub fn update_water_ripples(
    mut commands: Commands,
    mut ripples: Query<(Entity, &mut WaterRipple, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();

    for (entity, mut ripple, mut transform, mesh_material) in &mut ripples {
        ripple.lifetime += delta;
        if ripple.lifetime >= ripple.max_lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let t = ripple.lifetime / ripple.max_lifetime;

        // Grow from small to max_scale
        let scale = 10.0 + t * ripple.max_scale;
        transform.scale = Vec3::splat(scale);

        // Fade alpha: ramp up quickly then fade out
        let alpha = if t < 0.1 {
            t / 0.1
        } else {
            1.0 - (t - 0.1) / 0.9
        } * WATER_RIPPLE_ALPHA;

        if let Some(mat) = materials.get_mut(&mesh_material.0) {
            mat.base_color = Color::srgba(0.9, 0.95, 1.0, alpha);
        }
    }
}

// ===== Terrain Hazard Systems =====

/// Deals damage to any living unit inside the lava pool.
pub fn apply_lava_damage(
    time: Res<Time>,
    mut units: Query<(&Transform, &mut Health), Without<Corpse>>,
) {
    let damage = LAVA_DAMAGE_PER_SECOND * time.delta_secs();
    let lava_xz = Vec2::new(LAVA_POOL_POSITION.x, LAVA_POOL_POSITION.z);
    let radius_sq = LAVA_DAMAGE_RADIUS * LAVA_DAMAGE_RADIUS;

    for (transform, mut health) in &mut units {
        let unit_xz = Vec2::new(transform.translation.x, transform.translation.z);
        if unit_xz.distance_squared(lava_xz) <= radius_sq {
            health.take_damage(damage);
        }
    }
}

/// Applies a speed slow to units inside the water pool, removes it when they leave.
pub fn apply_water_slow(
    mut units: Query<(&Transform, &mut RoughTerrainModifier), Without<Corpse>>,
) {
    let water_xz = Vec2::new(WATER_POOL_POSITION.x, WATER_POOL_POSITION.z);
    let radius_sq = WATER_POOL_RADIUS * WATER_POOL_RADIUS;

    for (transform, mut terrain_mod) in &mut units {
        let unit_xz = Vec2::new(transform.translation.x, transform.translation.z);
        if unit_xz.distance_squared(water_xz) <= radius_sq {
            if terrain_mod.0 != WATER_SPEED_MODIFIER {
                terrain_mod.0 = WATER_SPEED_MODIFIER;
            }
        } else if terrain_mod.0 != 0.0 {
            terrain_mod.0 = 0.0;
        }
    }
}
