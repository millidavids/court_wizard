use bevy::prelude::*;

use crate::game::units::systems::create_sprite_material;

/// Loads a horizontal sprite sheet and creates one material per sprite variant.
///
/// Returns a `(mesh, materials)` tuple where the mesh is a rectangle sized to
/// a single sprite and each material samples the correct UV region.
pub(crate) fn preload_sprite_sheet<const N: usize>(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    asset_path: &'static str,
    sprite_width: f32,
    sprite_height: f32,
) -> (Handle<Mesh>, [Handle<StandardMaterial>; N]) {
    let texture: Handle<Image> = asset_server.load(asset_path);
    let mesh = meshes.add(Rectangle::new(sprite_width, sprite_height));

    let u_width = 1.0 / N as f32;
    let sprite_materials: [Handle<StandardMaterial>; N] = std::array::from_fn(|i| {
        let u_offset = i as f32 * u_width;
        create_sprite_material(
            materials,
            texture.clone(),
            Color::WHITE,
            Vec2::new(u_width, 1.0),
            Vec2::new(u_offset, 0.0),
        )
    });

    (mesh, sprite_materials)
}
