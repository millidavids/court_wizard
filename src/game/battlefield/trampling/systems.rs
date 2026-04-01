use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::components::TramplingOverlay;
use super::constants::*;
use super::resources::TramplingGrid;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::BATTLEFIELD_SIZE;
use crate::game::units::aerialist::components::Aerialist;
use crate::game::units::components::{Corpse, Health};

/// Nearest-neighbor sampler for the trampling texture (one pixel per grid cell).
fn nearest_sampler() -> ImageSampler {
    ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        ..default()
    })
}

/// Spawns the trampling overlay: a single quad covering the battlefield with
/// a runtime-generated texture (one pixel per grid cell).
pub fn spawn_trampling_overlay(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    grid: &TramplingGrid,
) {
    let size = grid.tiles_per_side as u32;

    let (r, g, b) = trampling_color_rgb();
    let pixel = [r, g, b, 0u8];
    let mut image = Image::new_fill(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &pixel,
        TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    );
    image.sampler = nearest_sampler();

    let image_handle = images.add(image);

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    let mesh = Plane3d::default()
        .mesh()
        .size(BATTLEFIELD_SIZE, BATTLEFIELD_SIZE);

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, TRAMPLING_OVERLAY_Y, 0.0),
        TramplingOverlay,
        OnGameplayScreen,
    ));
}

/// Tracks unit positions and increments trampling intensity when a unit
/// enters a new cell. Standing still on a cell does not increase trampling.
pub fn track_unit_trampling(
    units: Query<(Entity, &Transform), (With<Health>, Without<Corpse>, Without<Aerialist>)>,
    mut grid: ResMut<TramplingGrid>,
    mut last_cells: Local<std::collections::HashMap<Entity, usize>>,
    mut cleanup_counter: Local<u32>,
) {
    for (entity, transform) in &units {
        let pos = transform.translation;
        if let Some(idx) = grid.world_to_index(pos.x, pos.z) {
            let prev = last_cells.get(&entity).copied();
            if prev != Some(idx) {
                last_cells.insert(entity, idx);
                let v = &mut grid.values[idx];
                if *v < 1.0 {
                    *v = (*v + TRAMPLING_INCREMENT).min(1.0);
                    grid.dirty = true;
                }
            }
        }
    }

    // Clean up despawned entities every ~120 frames
    *cleanup_counter += 1;
    if *cleanup_counter >= 120 {
        *cleanup_counter = 0;
        let alive: std::collections::HashSet<Entity> = units.iter().map(|(e, _)| e).collect();
        last_cells.retain(|e, _| alive.contains(e));
    }
}

/// Periodically syncs the trampling grid values to the overlay texture by
/// building a fresh Image and replacing the material's texture handle.
/// Direct mutation of `image.data` doesn't reliably trigger GPU re-upload
/// in Bevy 0.17, so we replace the asset instead.
pub fn sync_trampling_texture(
    grid: Res<TramplingGrid>,
    overlay_query: Query<&MeshMaterial3d<StandardMaterial>, With<TramplingOverlay>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < TRAMPLING_SYNC_INTERVAL {
        return;
    }
    *timer -= TRAMPLING_SYNC_INTERVAL;

    if !grid.dirty {
        return;
    }

    let Ok(mesh_material) = overlay_query.single() else {
        return;
    };
    let Some(material) = materials.get_mut(&mesh_material.0) else {
        return;
    };

    if let Some(ref old_handle) = material.base_color_texture {
        images.remove(old_handle);
    }

    let size = grid.tiles_per_side as u32;
    let (r, g, b) = trampling_color_rgb();

    let pixel_count = grid.values.len();
    let mut data = Vec::with_capacity(pixel_count * 4);
    for value in &grid.values {
        let alpha = (*value * TRAMPLING_MAX_ALPHA * 255.0) as u8;
        data.push(r);
        data.push(g);
        data.push(b);
        data.push(alpha);
    }

    let mut image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    );
    image.sampler = nearest_sampler();

    material.base_color_texture = Some(images.add(image));
}

/// Marks the grid as clean after texture sync (runs after sync_trampling_texture).
pub fn clear_dirty_flag(mut grid: ResMut<TramplingGrid>) {
    grid.dirty = false;
}

/// Decays trampling between levels (grass grows back).
pub fn decay_trampling(mut grid: ResMut<TramplingGrid>) {
    grid.decay(TRAMPLING_DECAY_PER_LEVEL);
}

/// Resets trampling grid when exiting to menu.
pub fn reset_trampling(mut grid: ResMut<TramplingGrid>) {
    grid.reset();
}

/// Saves the current trampling grid to GameConfig before exiting InGame.
pub fn save_trampling_to_config(
    grid: Res<TramplingGrid>,
    mut config: ResMut<crate::config::GameConfig>,
) {
    config.saved_trampling = grid.to_saved();
}

/// Restores trampling grid from GameConfig on level load.
pub fn restore_trampling_from_config(
    mut grid: ResMut<TramplingGrid>,
    config: Res<crate::config::GameConfig>,
) {
    grid.restore_saved(&config.saved_trampling);
}

fn trampling_color_rgb() -> (u8, u8, u8) {
    let srgba = TRAMPLING_COLOR.to_srgba();
    (
        (srgba.red * 255.0) as u8,
        (srgba.green * 255.0) as u8,
        (srgba.blue * 255.0) as u8,
    )
}
