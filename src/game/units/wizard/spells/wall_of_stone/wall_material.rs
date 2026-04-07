use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

/// Custom material for Wall of Stone that renders the top face with battlefield
/// tile texture and side faces with procedural brown perlin noise, making the
/// wall look like a raised portion of the battlefield.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct WallOfStoneMaterial {
    /// x = tile_world_size, y = tile_count, z = tile_index, w = noise_intensity.
    #[uniform(0)]
    pub tile_params: Vec4,

    /// rgb = damage tint color, a = blend factor (0.0 = healthy, 1.0 = fully damaged).
    #[uniform(0)]
    pub damage_tint: Vec4,

    /// rgb = dark stone color for side faces and dirt border.
    #[uniform(0)]
    pub side_dark: Vec4,

    /// rgb = light stone color for side faces and dirt border.
    #[uniform(0)]
    pub side_light: Vec4,

    /// The battlefield tile sprite sheet texture.
    #[texture(1)]
    #[sampler(2)]
    pub base_texture: Handle<Image>,
}

impl Material for WallOfStoneMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/wall_of_stone.wgsl".into()
    }
}
