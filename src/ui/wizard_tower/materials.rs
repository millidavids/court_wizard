use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

/// Radial progress ring drawn as a UI material on spell graph nodes.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(super) struct RadialProgressMaterial {
    #[uniform(0)]
    pub data: RadialProgressData,
}

#[derive(ShaderType, Debug, Clone, Copy)]
pub(super) struct RadialProgressData {
    pub fill_color: LinearRgba,
    pub bg_color: LinearRgba,
    /// Progress in [0.0, 1.0].
    pub progress: f32,
    /// Ring width as a fraction of the node radius (0.0–1.0).
    pub ring_width: f32,
}

impl UiMaterial for RadialProgressMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/radial_progress.wgsl".into()
    }
}

/// Concentric rings drawn on insight bonus constellation nodes.
/// Shows 5 rings: filled (permanent), pending (allocated), and empty.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(super) struct ConcentricRingsMaterial {
    #[uniform(0)]
    pub data: ConcentricRingsData,
}

#[derive(ShaderType, Debug, Clone, Copy)]
pub(super) struct ConcentricRingsData {
    pub fill_color: LinearRgba,
    pub bg_color: LinearRgba,
    pub pending_color: LinearRgba,
    /// Number of permanently filled rings (0-5).
    pub filled: f32,
    /// Number of additionally pending rings (0-5).
    pub pending: f32,
    /// Total number of rings.
    pub total: f32,
}

impl UiMaterial for ConcentricRingsMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/concentric_rings.wgsl".into()
    }
}

/// Twinkling starry sky background drawn as a UI material on the spell graph area.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(super) struct StarSkyMaterial {
    #[uniform(0)]
    pub data: StarSkyData,
}

#[derive(ShaderType, Debug, Clone, Copy)]
pub(super) struct StarSkyData {
    pub base_color: LinearRgba,
    pub time: f32,
    pub zoom: f32,
    pub pan_offset: Vec2,
}

impl UiMaterial for StarSkyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/star_sky.wgsl".into()
    }
}
