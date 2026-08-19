use bevy::material::AlphaMode;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

/// Material for terrain vegetation sprites with wind sway animation.
///
/// Replaces `StandardMaterial` for trees, bushes, and flora. Replicates the
/// unlit alpha-masked texture sampling and adds time-based UV distortion in
/// the fragment shader to simulate wind.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct WindSwayMaterial {
    /// Base color tint (multiplied with texture sample).
    #[uniform(0)]
    pub base_color: LinearRgba,

    /// UV transform packed as `(scale_x, scale_y, offset_x, offset_y)`.
    #[uniform(0)]
    pub uv_params: Vec4,

    /// Sway parameters: `(time, amplitude, speed, phase_offset)`.
    /// `time` is updated each frame by `update_wind_sway_time`.
    #[uniform(0)]
    pub sway_params: Vec4,

    /// The sprite sheet texture.
    #[texture(1)]
    #[sampler(2)]
    pub base_texture: Handle<Image>,
}

impl Material for WindSwayMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/wind_sway.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(0.5)
    }
}
