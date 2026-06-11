use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    render::{
        extract_component::{ComponentUniforms, DynamicUniformIndex},
        render_graph::{NodeRunError, RenderGraphContext, RenderLabel, ViewNode},
        render_resource::*,
        renderer::RenderContext,
        view::ViewTarget,
    },
};

use super::super::components::{
    HeatDistortionSettings, LensingSettings, TeleportDistortionSettings,
};
use super::accessibility_pipeline::TeleportDistortionPipeline;
use super::crt_pipeline::{HeatDistortionPipeline, LensingPipeline};

// --- Gravitational Lensing render node ---

#[derive(Default)]
pub(crate) struct LensingNode;

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

// --- Heat Distortion render node ---

#[derive(Default)]
pub(crate) struct HeatDistortionNode;

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

// --- Teleport Distortion render node ---

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct TeleportDistortionLabel;

#[derive(Default)]
pub(crate) struct TeleportDistortionNode;

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
