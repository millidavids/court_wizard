use bevy::material::AlphaMode;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

/// Custom fire particle material with procedural noise, color ramp, and dissolve.
///
/// All fire smoke particles share a single handle for GPU batching (1 draw call).
/// The shader uses `world_position` as a per-particle noise seed so each particle
/// shows a unique animated fire pattern. The `time` uniform drives scrolling animation.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(crate) struct FireParticleMaterial {
    /// Global elapsed time (updated each frame by `update_fire_particle_time`).
    #[uniform(0)]
    pub time: f32,
}

impl Material for FireParticleMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/fire_particle.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

/// Smoke particle material with circular radial falloff.
///
/// All smoke particles share a single handle for GPU batching.
/// The shader applies a soft circular mask so smoke looks round, not square.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(crate) struct SmokeParticleMaterial {
    /// Base color and alpha of the smoke.
    #[uniform(0)]
    pub color: LinearRgba,
}

impl Material for SmokeParticleMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/smoke_particle.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
