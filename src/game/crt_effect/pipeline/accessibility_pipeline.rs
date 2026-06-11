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

use super::super::components::{
    ColorblindCorrectionSettings, HighContrastSettings, TeleportDistortionSettings,
};

pub(crate) const TELEPORT_DISTORTION_SHADER_PATH: &str = "shaders/teleport_distortion.wgsl";
pub(crate) const COLORBLIND_SHADER_PATH: &str = "shaders/colorblind_correction.wgsl";
pub(crate) const HIGH_CONTRAST_SHADER_PATH: &str = "shaders/high_contrast.wgsl";

#[derive(Resource)]
pub(crate) struct TeleportDistortionPipeline {
    pub(crate) layout: BindGroupLayout,
    pub(crate) sampler: Sampler,
    pub(crate) pipeline_id: CachedRenderPipelineId,
}

pub(crate) fn init_teleport_distortion_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout_desc = BindGroupLayoutDescriptor::new(
        "teleport_distortion_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<TeleportDistortionSettings>(true),
            ),
        ),
    );

    let sampler = render_device.create_sampler(&SamplerDescriptor::default());
    let shader = asset_server.load(TELEPORT_DISTORTION_SHADER_PATH);
    let vertex_state = fullscreen_shader.to_vertex_state();

    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("teleport_distortion_pipeline".into()),
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

    commands.insert_resource(TeleportDistortionPipeline {
        layout: pipeline_cache.get_bind_group_layout(&layout_desc),
        sampler,
        pipeline_id,
    });
}

#[derive(Resource)]
pub(crate) struct ColorblindCorrectionPipeline {
    pub(crate) layout: BindGroupLayout,
    pub(crate) sampler: Sampler,
    pub(crate) pipeline_id: CachedRenderPipelineId,
}

pub(crate) fn init_colorblind_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout_desc = BindGroupLayoutDescriptor::new(
        "colorblind_correction_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<ColorblindCorrectionSettings>(true),
            ),
        ),
    );

    let sampler = render_device.create_sampler(&SamplerDescriptor::default());
    let shader = asset_server.load(COLORBLIND_SHADER_PATH);
    let vertex_state = fullscreen_shader.to_vertex_state();

    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("colorblind_correction_pipeline".into()),
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

    commands.insert_resource(ColorblindCorrectionPipeline {
        layout: pipeline_cache.get_bind_group_layout(&layout_desc),
        sampler,
        pipeline_id,
    });
}

#[derive(Resource)]
pub(crate) struct HighContrastPipeline {
    pub(crate) layout: BindGroupLayout,
    pub(crate) sampler: Sampler,
    pub(crate) pipeline_id: CachedRenderPipelineId,
}

pub(crate) fn init_high_contrast_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout_desc = BindGroupLayoutDescriptor::new(
        "high_contrast_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<HighContrastSettings>(true),
            ),
        ),
    );

    let sampler = render_device.create_sampler(&SamplerDescriptor::default());
    let shader = asset_server.load(HIGH_CONTRAST_SHADER_PATH);
    let vertex_state = fullscreen_shader.to_vertex_state();

    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("high_contrast_pipeline".into()),
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

    commands.insert_resource(HighContrastPipeline {
        layout: pipeline_cache.get_bind_group_layout(&layout_desc),
        sampler,
        pipeline_id,
    });
}
