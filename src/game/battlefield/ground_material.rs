use bevy::material::AlphaMode;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

/// Custom material for battlefield ground tiles with procedural noise overlay.
///
/// Uses world-space simplex noise to break up the flat green with subtle
/// color variation. `noise_intensity` controls the strength — higher under
/// tree concentrations for a more earthy, varied forest floor.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct GroundMaterial {
    /// x = noise intensity (0.0 = no noise, 1.0 = full effect).
    #[uniform(0)]
    pub params: Vec4,

    /// The tile sprite sheet texture.
    #[texture(1)]
    #[sampler(2)]
    pub base_texture: Handle<Image>,
}

impl Material for GroundMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ground_noise.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

/// Procedural noise underlay placed beneath the wall floor.
/// Used for both reddish-brown stone (tunnels) and sandy (pond area) variants.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct StoneNoiseMaterial {
    /// rgb = dark color.
    #[uniform(0)]
    pub dark_color: Vec4,
    /// rgb = light color.
    #[uniform(0)]
    pub light_color: Vec4,
    /// Blend parameters:
    /// - mode 0 (circular): x=center_x, y=center_z, z=inner_radius, w=outer_radius
    /// - mode 1 (wall edge): x=wall_x, y=fade_start_distance, z=fade_end_distance, w=unused
    #[uniform(0)]
    pub blend_params: Vec4,
    /// x = blend mode (0.0 = circular, 1.0 = wall edge).
    #[uniform(0)]
    pub blend_mode: Vec4,
}

impl Material for StoneNoiseMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/stone_noise.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
