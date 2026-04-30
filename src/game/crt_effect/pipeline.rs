use bevy::{
    core_pipeline::FullscreenShader,
    ecs::query::QueryItem,
    prelude::*,
    render::{
        extract_component::{ComponentUniforms, DynamicUniformIndex},
        render_graph::{NodeRunError, RenderGraphContext, RenderLabel, ViewNode},
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::ViewTarget,
    },
};

use super::components::{
    ColorblindCorrectionSettings, CrtEffectSettings, HeatDistortionSettings, HighContrastSettings,
    LensingSettings, TeleportDistortionSettings,
};
use crate::config::GameConfig;

const CRT_SHADER_PATH: &str = "shaders/crt_effect.wgsl";
const LENSING_SHADER_PATH: &str = "shaders/gravitational_lensing.wgsl";
const HEAT_DISTORTION_SHADER_PATH: &str = "shaders/heat_distortion.wgsl";
const TELEPORT_DISTORTION_SHADER_PATH: &str = "shaders/teleport_distortion.wgsl";
const COLORBLIND_SHADER_PATH: &str = "shaders/colorblind_correction.wgsl";
const HIGH_CONTRAST_SHADER_PATH: &str = "shaders/high_contrast.wgsl";

#[derive(Resource)]
pub(super) struct CrtEffectPipeline {
    pub layout: BindGroupLayout,
    pub sampler: Sampler,
    pub pipeline_id: CachedRenderPipelineId,
}

pub(super) fn init_crt_pipeline(
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

// --- Gravitational Lensing render node ---

#[derive(Default)]
pub(super) struct LensingNode;

impl ViewNode for LensingNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static LensingSettings,
        &'static DynamicUniformIndex<LensingSettings>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, settings, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        if settings.lensing_count < 0.5 {
            return Ok(());
        }

        let lensing_pipeline = world.resource::<LensingPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(lensing_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<LensingSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            "lensing_bind_group",
            &lensing_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &lensing_pipeline.sampler,
                settings_binding.clone(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("lensing_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
pub(super) struct LensingPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

pub(super) fn init_lensing_pipeline(
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

// --- Heat Distortion render node ---

#[derive(Default)]
pub(super) struct HeatDistortionNode;

impl ViewNode for HeatDistortionNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static HeatDistortionSettings,
        &'static DynamicUniformIndex<HeatDistortionSettings>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, settings, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        // Skip the entire render pass when no walls are active
        if settings.count < 0.5 {
            return Ok(());
        }

        let heat_pipeline = world.resource::<HeatDistortionPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(heat_pipeline.pipeline_id) else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<HeatDistortionSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            "heat_distortion_bind_group",
            &heat_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &heat_pipeline.sampler,
                settings_binding.clone(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("heat_distortion_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
pub(super) struct HeatDistortionPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

pub(super) fn init_heat_distortion_pipeline(
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

// --- Teleport Distortion render node ---

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(super) struct TeleportDistortionLabel;

#[derive(Default)]
pub(super) struct TeleportDistortionNode;

impl ViewNode for TeleportDistortionNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static TeleportDistortionSettings,
        &'static DynamicUniformIndex<TeleportDistortionSettings>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, settings, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        // Skip the entire render pass when no distortion points are active
        if settings.count < 0.5 {
            return Ok(());
        }

        let teleport_pipeline = world.resource::<TeleportDistortionPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(teleport_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<TeleportDistortionSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            "teleport_distortion_bind_group",
            &teleport_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &teleport_pipeline.sampler,
                settings_binding.clone(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("teleport_distortion_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
pub(super) struct TeleportDistortionPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

pub(super) fn init_teleport_distortion_pipeline(
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

pub(super) fn update_crt_time(time: Res<Time>, mut query: Query<&mut CrtEffectSettings>) {
    for mut settings in &mut query {
        settings.time = time.elapsed_secs();
    }
}

// --- Colorblind Correction render node ---

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(super) struct ColorblindCorrectionLabel;

#[derive(Default)]
pub(super) struct ColorblindCorrectionNode;

impl ViewNode for ColorblindCorrectionNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ColorblindCorrectionSettings,
        &'static DynamicUniformIndex<ColorblindCorrectionSettings>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, settings, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        // Skip the entire render pass when disabled
        if settings.enabled < 0.5 {
            return Ok(());
        }

        let colorblind_pipeline = world.resource::<ColorblindCorrectionPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(colorblind_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<ColorblindCorrectionSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            "colorblind_correction_bind_group",
            &colorblind_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &colorblind_pipeline.sampler,
                settings_binding.clone(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("colorblind_correction_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
pub(super) struct ColorblindCorrectionPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

pub(super) fn init_colorblind_pipeline(
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

/// Syncs GameConfig colorblind settings to the camera's ColorblindCorrectionSettings component.
/// Uses Local to track previous values and skip no-op updates (avoids unnecessary GPU re-uploads
/// when unrelated GameConfig fields like volume change).
pub(super) fn sync_colorblind_settings(
    config: Res<GameConfig>,
    mut query: Query<&mut ColorblindCorrectionSettings>,
    mut last: Local<(crate::config::ColorblindType, u32)>,
) {
    let strength_bits = config.colorblind_strength.to_bits();
    if last.0 == config.colorblind_type && last.1 == strength_bits {
        return;
    }
    *last = (config.colorblind_type, strength_bits);
    let new_settings =
        ColorblindCorrectionSettings::for_type(config.colorblind_type, config.colorblind_strength);
    for mut settings in &mut query {
        *settings = new_settings;
    }
}

/// Syncs the CRT effect enabled state from GameConfig to the camera component.
/// Also zeroes barrel_distortion when disabled so the shader samples undistorted UVs
/// and cursor correction (which checks `is_barrel_active()`) is skipped.
pub(super) fn sync_crt_enabled(
    config: Res<GameConfig>,
    mut query: Query<&mut CrtEffectSettings>,
    mut last: Local<bool>,
) {
    if *last == config.crt_enabled {
        return;
    }
    *last = config.crt_enabled;
    for mut settings in &mut query {
        if config.crt_enabled {
            settings.enabled = 1.0;
            settings.barrel_distortion = super::constants::DEFAULT_BARREL_DISTORTION;
        } else {
            settings.enabled = 0.0;
            settings.barrel_distortion = 0.0;
        }
    }
}

// --- High Contrast render node ---

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(super) struct HighContrastLabel;

#[derive(Default)]
pub(super) struct HighContrastNode;

impl ViewNode for HighContrastNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static HighContrastSettings,
        &'static DynamicUniformIndex<HighContrastSettings>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, settings, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        if settings.enabled < 0.5 {
            return Ok(());
        }

        let hc_pipeline = world.resource::<HighContrastPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(hc_pipeline.pipeline_id) else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<HighContrastSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            "high_contrast_bind_group",
            &hc_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &hc_pipeline.sampler,
                settings_binding.clone(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("high_contrast_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
pub(super) struct HighContrastPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

pub(super) fn init_high_contrast_pipeline(
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

/// Syncs high contrast settings from GameConfig to the camera component.
pub(super) fn sync_high_contrast(
    config: Res<GameConfig>,
    mut query: Query<&mut HighContrastSettings>,
    mut last: Local<u32>,
) {
    let bits = config.high_contrast_strength.to_bits();
    if *last == bits {
        return;
    }
    *last = bits;
    let enabled = if config.high_contrast_strength > 0.01 {
        1.0
    } else {
        0.0
    };
    for mut settings in &mut query {
        settings.strength = config.high_contrast_strength;
        settings.enabled = enabled;
    }
}

/// Sets CRT flicker intensity to zero when reduce_flashes is enabled.
pub(super) fn sync_flicker_intensity(
    config: Res<GameConfig>,
    mut query: Query<&mut CrtEffectSettings>,
    mut last: Local<bool>,
) {
    if *last == config.reduce_flashes {
        return;
    }
    *last = config.reduce_flashes;
    let intensity = if config.reduce_flashes {
        0.0
    } else {
        super::constants::DEFAULT_FLICKER_INTENSITY
    };
    for mut settings in &mut query {
        settings.flicker_intensity = intensity;
    }
}
