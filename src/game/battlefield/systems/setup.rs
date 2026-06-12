use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use rand::Rng;

use crate::game::battlefield::components::{
    Battlefield, BattlefieldAssets, Castle, LavaPool, LeftWall, RightWall, WallFloor,
    WaterRippleAssets,
};
use crate::game::battlefield::constants::*;
use crate::game::battlefield::ground_material::{GroundMaterial, StoneNoiseMaterial};
use crate::game::components::OnGameplayScreen;
use crate::game::constants::*;

/// Pre-loads battlefield textures at startup.
pub(crate) fn load_battlefield_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BattlefieldAssets {
        castle_wall: asset_server.load("images/castle_wall.png"),
        right_wall: asset_server.load("images/static_sprites/right_wall.png"),
        left_wall: asset_server.load("images/static_sprites/left_wall.png"),
        wall_floor: asset_server.load("images/static_sprites/wall_floor.png"),
        battlefield_tiles: asset_server.load("images/sprite_sheets/battlefield_tiles.png"),
    });
}

/// Sets up the battlefield and castle when entering the InGame state.
///
/// Spawns the battlefield ground plane, castle wall image, and point light in 3D space.
#[allow(clippy::too_many_arguments)]
pub fn setup_battlefield(
    rng: &mut impl Rng,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    ground_materials: &mut Assets<GroundMaterial>,
    stone_materials: &mut Assets<StoneNoiseMaterial>,
    battlefield_assets: &BattlefieldAssets,
    // World-space transform composed onto every spawned visual. Single-player
    // and the multiplayer host pass `Transform::IDENTITY`. The multiplayer
    // guest passes a 180° Y-rotation so their copy of the battlefield (which
    // is purely visual — units and terrain remain in shared world coords) is
    // mirrored to match their mirrored camera.
    origin_transform: Transform,
) {
    // Add a light source so we can see 3D objects
    commands.spawn((
        PointLight {
            intensity: 2_000_000.0,
            shadows_enabled: false,
            ..default()
        },
        origin_transform * Transform::from_xyz(0.0, 1000.0, 0.0),
        OnGameplayScreen,
    ));

    // Spawn battlefield as a grid of textured ground tiles
    spawn_ground_tiles(
        rng,
        commands,
        meshes,
        ground_materials,
        battlefield_assets,
        origin_transform,
    );

    // Spawn castle wall as a textured plane the wizard stands on
    spawn_castle_wall(
        commands,
        meshes,
        materials,
        battlefield_assets,
        CASTLE_POSITION,
        CASTLE_ROTATION_DEGREES,
        OnGameplayScreen,
        origin_transform,
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
        origin_transform
            * Transform::from_translation(RIGHT_WALL_POSITION).with_rotation(
                Quat::from_rotation_y(RIGHT_WALL_ROTATION_DEGREES.to_radians()),
            ),
    );
    spawn_wall_backdrop(
        commands,
        meshes,
        materials,
        battlefield_assets.left_wall.clone(),
        LEFT_WALL_WIDTH,
        LEFT_WALL_HEIGHT,
        origin_transform * Transform::from_translation(LEFT_WALL_POSITION),
        LeftWall,
        0.0,
    );

    // Noise underlays beneath the wall floor — transparent areas in
    // wall_floor.png reveal procedural noise instead of grass.
    {
        // Sand underlay (circular blend around the pond)
        let sand_size = WATER_POOL_RADIUS * 3.5;
        let sand_mesh = Rectangle::new(sand_size, sand_size);
        // +100.0 / -100.0 offsets nudge the sand patch slightly toward the castle
        // to align the circular blend center with the visual pond art.
        let sand_material = stone_materials.add(StoneNoiseMaterial {
            dark_color: Vec4::new(0.55, 0.45, 0.30, 1.0),
            light_color: Vec4::new(0.75, 0.65, 0.48, 1.0),
            blend_params: Vec4::new(
                WATER_POOL_POSITION.x + 100.0,
                WATER_POOL_POSITION.z - 100.0,
                WATER_POOL_RADIUS * 0.3,
                WATER_POOL_RADIUS * 2.0,
            ),
            blend_mode: Vec4::new(0.0, 0.0, 0.0, 0.0),
        });
        commands.spawn((
            Mesh3d(meshes.add(sand_mesh)),
            MeshMaterial3d(sand_material),
            origin_transform
                * Transform::from_xyz(
                    WATER_POOL_POSITION.x + 100.0,
                    0.5,
                    WATER_POOL_POSITION.z - 100.0,
                )
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            OnGameplayScreen,
        ));

        // Stone underlay (diagonal-line blend following the wall floor art border)
        // Oversized plane covering full battlefield height so it reaches the map edge.
        let stone_width = WALL_FLOOR_DEPTH + 2000.0;
        let stone_length = BATTLEFIELD_SIZE + 2000.0;
        let stone_mesh = Rectangle::new(stone_width, stone_length);
        let stone_dark = STONE_COLOR_DARK.to_srgba();
        let stone_light = STONE_COLOR_LIGHT.to_srgba();
        let stone_material = stone_materials.add(StoneNoiseMaterial {
            dark_color: Vec4::new(stone_dark.red, stone_dark.green, stone_dark.blue, 1.0),
            light_color: Vec4::new(stone_light.red, stone_light.green, stone_light.blue, 1.0),
            blend_params: Vec4::new(
                0.0,   // unused (line is hardcoded in shader)
                50.0,  // fade_start (tight to the art edge)
                800.0, // fade_end (blend distance into grass)
                0.0,
            ),
            blend_mode: Vec4::new(1.0, 0.0, 0.0, 0.0),
        });
        commands.spawn((
            Mesh3d(meshes.add(stone_mesh)),
            MeshMaterial3d(stone_material),
            // Centered at Z=-1500 (further from camera than wall_floor at Z=-1000) so
            // the transparent wall_floor sorts on top of this opaque underlay.
            origin_transform
                * Transform::from_xyz(WALL_FLOOR_POSITION.x, 0.5, -1500.0)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            OnGameplayScreen,
        ));
    }

    // Spawn floor between the right and left walls (laid flat, facing up).
    // AlphaMode::Blend so translucent edges in the art blend properly.
    // The stone/sand underlays are opaque (blend toward grass green at edges),
    // so this transparent layer always renders on top of them.
    {
        let mesh = Rectangle::new(WALL_FLOOR_DEPTH, WALL_FLOOR_LENGTH);
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(battlefield_assets.wall_floor.clone()),
            base_color: Color::WHITE,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        });
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material),
            origin_transform
                * Transform::from_translation(WALL_FLOOR_POSITION)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            WallFloor,
            OnGameplayScreen,
        ));
    }

    // Spawn lava pool marker (drives fire/smoke/spark effects)
    commands.spawn((
        origin_transform * Transform::from_translation(LAVA_POOL_POSITION),
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
#[allow(clippy::too_many_arguments)]
pub fn spawn_castle_wall<M: Component + Clone>(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    battlefield_assets: &BattlefieldAssets,
    castle_position: Vec3,
    rotation_degrees: f32,
    screen_marker: M,
    origin_transform: Transform,
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
        origin_transform
            * Transform::from_translation(castle_position)
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
        vec![[u_max, 0.0], [u_min, 0.0], [u_min, 1.0], [u_max, 1.0]],
    );
    mesh.insert_indices(Indices::U32(vec![0, 2, 1, 0, 3, 2]));
    mesh
}

/// Picks a weighted-random tile index. Base tiles are heavily favored over flavor tiles.
fn pick_weighted_tile(rng: &mut impl Rng) -> usize {
    let roll: u32 = rng.random_range(0..TILE_TOTAL_WEIGHT);
    let base_total = TILE_BASE_COUNT as u32 * TILE_BASE_WEIGHT;
    if roll < base_total {
        (roll / TILE_BASE_WEIGHT) as usize
    } else {
        TILE_BASE_COUNT + ((roll - base_total) / TILE_FLAVOR_WEIGHT) as usize
    }
}

/// Spawns a grid of ground tiles covering the battlefield.
fn spawn_ground_tiles(
    rng: &mut impl Rng,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<GroundMaterial>,
    battlefield_assets: &BattlefieldAssets,
    origin_transform: Transform,
) {
    let half = BATTLEFIELD_SIZE / 2.0;
    let tiles_per_side = (BATTLEFIELD_SIZE / TILE_WORLD_SIZE) as usize;

    // Pre-create one mesh handle per tile variant (shared across all tiles of that type)
    let tile_meshes: Vec<Handle<Mesh>> = (0..TILE_COUNT)
        .map(|i| meshes.add(create_tile_mesh(TILE_WORLD_SIZE, i, TILE_COUNT)))
        .collect();

    // Single shared material — the shader computes intensity from world Z position.
    let tile_material = materials.add(GroundMaterial {
        params: Vec4::new(
            GROUND_NOISE_BASE,
            GROUND_NOISE_FOREST,
            GROUND_NOISE_FOREST_Z,
            GROUND_NOISE_BLEND_RANGE,
        ),
        base_texture: battlefield_assets.battlefield_tiles.clone(),
    });

    for row in 0..tiles_per_side {
        for col in 0..tiles_per_side {
            let x = -half + TILE_WORLD_SIZE * (col as f32 + 0.5);
            let z = -half + TILE_WORLD_SIZE * (row as f32 + 0.5);
            let tile_index = pick_weighted_tile(rng);

            commands.spawn((
                Mesh3d(tile_meshes[tile_index].clone()),
                MeshMaterial3d(tile_material.clone()),
                origin_transform * Transform::from_xyz(x, -1.0, z),
                Battlefield,
                OnGameplayScreen,
            ));
        }
    }
}
