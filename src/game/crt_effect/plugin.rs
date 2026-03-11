use bevy::{
    core_pipeline::{
        FullscreenShader,
        core_3d::graph::{Core3d, Node3d},
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        RenderApp, RenderStartup,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponentPlugin, UniformComponentPlugin,
        },
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::ViewTarget,
    },
    ui_render::graph::NodeUi,
};

use super::components::{ChannelChangeTimer, CrtEffectSettings, DesaturationTimer, HeatDistortionSettings, LensingSettings};
use super::messages::{ChannelChangeMessage, ScreenDesaturateMessage};
use super::systems::{
    CorrectedCursorPosition, RawCursorPosition, animate_channel_change, animate_desaturation,
    correct_cursor_for_barrel_distortion, correct_ui_interaction_for_barrel,
    handle_channel_change_message, handle_desaturation_message, update_heat_distortion_positions,
    update_lensing_positions,
};
use crate::state::AppState;

const CRT_SHADER_PATH: &str = "shaders/crt_effect.wgsl";
const LENSING_SHADER_PATH: &str = "shaders/gravitational_lensing.wgsl";
const HEAT_DISTORTION_SHADER_PATH: &str = "shaders/heat_distortion.wgsl";

pub(crate) struct CrtEffectPlugin;

impl Plugin for CrtEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<CrtEffectSettings>::default(),
            UniformComponentPlugin::<CrtEffectSettings>::default(),
            ExtractComponentPlugin::<LensingSettings>::default(),
            UniformComponentPlugin::<LensingSettings>::default(),
            ExtractComponentPlugin::<HeatDistortionSettings>::default(),
            UniformComponentPlugin::<HeatDistortionSettings>::default(),
        ));

        app.init_resource::<RawCursorPosition>();
        app.init_resource::<CorrectedCursorPosition>();
        app.add_message::<ChannelChangeMessage>();
        app.add_message::<ScreenDesaturateMessage>();

        app.add_systems(Update, update_crt_time);
        app.add_systems(Update, handle_channel_change_message);
        app.add_systems(
            Update,
            animate_channel_change.run_if(resource_exists::<ChannelChangeTimer>),
        );
        app.add_systems(Update, handle_desaturation_message);
        app.add_systems(
            Update,
            animate_desaturation.run_if(resource_exists::<DesaturationTimer>),
        );
        app.add_systems(
            Update,
            (
                update_lensing_positions,
                update_heat_distortion_positions,
            )
                .run_if(in_state(AppState::InGame)),
        );

        // Correct cursor position for barrel distortion before any game systems read it.
        // Runs in PreUpdate so all downstream systems (spells, UI picking, input)
        // automatically get the corrected position.
        app.add_systems(PreUpdate, correct_cursor_for_barrel_distortion);

        // Correct UI Interaction components after Bevy's ui_focus_system has run.
        // ui_focus_system reads window.physical_cursor_position() (raw OS cursor),
        // so we re-do the hit testing with barrel-corrected coordinates and override
        // the Interaction values it set.
        app.add_systems(
            PreUpdate,
            correct_ui_interaction_for_barrel
                .after(bevy::ui::UiSystems::Focus)
                .run_if(|q: Query<&CrtEffectSettings>| {
                    q.single().is_ok_and(|s| s.is_barrel_active())
                }),
        );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(
            RenderStartup,
            (init_crt_pipeline, init_lensing_pipeline, init_heat_distortion_pipeline),
        );

        render_app
            .add_render_graph_node::<ViewNodeRunner<LensingNode>>(Core3d, LensingLabel)
            .add_render_graph_node::<ViewNodeRunner<HeatDistortionNode>>(Core3d, HeatDistortionLabel)
            .add_render_graph_node::<ViewNodeRunner<CrtEffectNode>>(Core3d, CrtEffectLabel)
            .add_render_graph_edges(
                Core3d,
                (NodeUi::UiPass, LensingLabel, HeatDistortionLabel, CrtEffectLabel, Node3d::Upscaling),
            );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct LensingLabel;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct HeatDistortionLabel;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct CrtEffectLabel;

#[derive(Default)]
struct CrtEffectNode;

impl ViewNode for CrtEffectNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static CrtEffectSettings,
        &'static DynamicUniformIndex<CrtEffectSettings>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, _settings, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let crt_pipeline = world.resource::<CrtEffectPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(crt_pipeline.pipeline_id) else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<CrtEffectSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            "crt_effect_bind_group",
            &crt_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &crt_pipeline.sampler,
                settings_binding.clone(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("crt_effect_pass"),
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
struct CrtEffectPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

fn init_crt_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = render_device.create_bind_group_layout(
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
        layout: vec![layout.clone()],
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
        layout,
        sampler,
        pipeline_id,
    });
}

// --- Gravitational Lensing render node ---

#[derive(Default)]
struct LensingNode;

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
        (view_target, _settings, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
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
struct LensingPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

fn init_lensing_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = render_device.create_bind_group_layout(
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
        layout: vec![layout.clone()],
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
        layout,
        sampler,
        pipeline_id,
    });
}

// --- Heat Distortion render node ---

#[derive(Default)]
struct HeatDistortionNode;

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
struct HeatDistortionPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

fn init_heat_distortion_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = render_device.create_bind_group_layout(
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
        layout: vec![layout.clone()],
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
        layout,
        sampler,
        pipeline_id,
    });
}

fn update_crt_time(time: Res<Time>, mut query: Query<&mut CrtEffectSettings>) {
    for mut settings in &mut query {
        settings.time = time.elapsed_secs();
    }
}
