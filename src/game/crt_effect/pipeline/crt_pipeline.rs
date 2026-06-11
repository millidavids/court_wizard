use bevy::{
    core_pipeline::FullscreenShader,
    prelude::*,
    render::{
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::RenderDevice,
    },
};

use super::super::components::{CrtEffectSettings, HeatDistortionSettings, LensingSettings};

pub(crate) const CRT_SHADER_PATH: &str = "shaders/crt_effect.wgsl";
pub(crate) const LENSING_SHADER_PATH: &str = "shaders/gravitational_lensing.wgsl";
pub(crate) const HEAT_DISTORTION_SHADER_PATH: &str = "shaders/heat_distortion.wgsl";

#[derive(Resource)]
pub(crate) struct CrtEffectPipeline {
    pub layout: BindGroupLayout,
    pub sampler: Sampler,
    pub pipeline_id: CachedRenderPipelineId,
}

pub(crate) fn init_crt_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout_desc = BindGroupLayoutDescriptor::new(
        "crt_effect_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<CrtEffectSettings>(true),
            ),
        ),
    );

    let sampler = render_device.create_sampler(&SamplerDescriptor::default());
    let shader = asset_server.load(CRT_SHADER_PATH);
    let vertex_state = fullscreen_shader.to_vertex_state();

    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("crt_effect_pipeline".into()),
        layout: vec![layout_desc.clone()],
        vertex: vertex_state,
        fragment: Some(FragmentState {
            shader,
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });

    commands.insert_resource(CrtEffectPipeline {
        layout: pipeline_cache.get_bind_group_layout(&layout_desc),
        sampler,
        pipeline_id,
    });
}

#[derive(Resource)]
pub(crate) struct LensingPipeline {
    pub(crate) layout: BindGroupLayout,
    pub(crate) sampler: Sampler,
    pub(crate) pipeline_id: CachedRenderPipelineId,
}

pub(crate) fn init_lensing_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout_desc = BindGroupLayoutDescriptor::new(
        "lensing_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<LensingSettings>(true),
            ),
        ),
    );

    let sampler = render_device.create_sampler(&SamplerDescriptor::default());
    let shader = asset_server.load(LENSING_SHADER_PATH);
    let vertex_state = fullscreen_shader.to_vertex_state();

    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("lensing_pipeline".into()),
        layout: vec![layout_desc.clone()],
        vertex: vertex_state,
        fragment: Some(FragmentState {
            shader,
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });

    commands.insert_resource(LensingPipeline {
        layout: pipeline_cache.get_bind_group_layout(&layout_desc),
        sampler,
        pipeline_id,
    });
}

#[derive(Resource)]
pub(crate) struct HeatDistortionPipeline {
    pub(crate) layout: BindGroupLayout,
    pub(crate) sampler: Sampler,
    pub(crate) pipeline_id: CachedRenderPipelineId,
}

pub(crate) fn init_heat_distortion_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout_desc = BindGroupLayoutDescriptor::new(
        "heat_distortion_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<HeatDistortionSettings>(true),
            ),
        ),
    );

    let sampler = render_device.create_sampler(&SamplerDescriptor::default());
    let shader = asset_server.load(HEAT_DISTORTION_SHADER_PATH);
    let vertex_state = fullscreen_shader.to_vertex_state();

    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("heat_distortion_pipeline".into()),
        layout: vec![layout_desc.clone()],
        vertex: vertex_state,
        fragment: Some(FragmentState {
            shader,
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });

    commands.insert_resource(HeatDistortionPipeline {
        layout: pipeline_cache.get_bind_group_layout(&layout_desc),
        sampler,
        pipeline_id,
    });
}
