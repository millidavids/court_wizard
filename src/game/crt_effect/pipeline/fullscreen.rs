//! Shared machinery for the CRT chain's fullscreen post-process passes.
//!
//! Every pass in the chain does the same thing: sample the view's current colour
//! texture, run one fullscreen triangle through a fragment shader with a single
//! uniform struct bound, and write the result back. Only the shader, the uniform
//! type, and the "is there anything to draw this frame" test differ — those live
//! in the [`FullscreenEffect`] impls in `effects.rs`, and everything else lives
//! here exactly once.

use core::marker::PhantomData;

use bevy::{
    core_pipeline::FullscreenShader,
    prelude::*,
    render::{
        extract_component::{ComponentUniforms, DynamicUniformIndex},
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            // `encase`'s own module layout calls this "internal", but it is the
            // only path Bevy re-exports and `DynamicUniformBuffer::binding()`
            // requires the bound, so a generic pass cannot avoid naming it.
            // Bevy's own `bevy_render::uniform` imports it exactly this way.
            encase::internal::WriteInto,
            *,
        },
        renderer::{RenderContext, RenderDevice, ViewQuery},
        view::ViewTarget,
    },
};

/// One fullscreen post-process pass in the CRT chain.
///
/// Implemented by the settings component that drives the pass — the same type
/// that is extracted to the render world and uploaded as the shader's uniform.
/// `ShaderType + WriteInto` is what lets that uniform be bound.
pub(crate) trait FullscreenEffect: Component + ShaderType + WriteInto {
    /// Base debug name for this pass. The layout, pipeline, bind group and
    /// render pass each get their own suffixed label derived from it, so a
    /// graphics-debugger capture names them individually.
    const LABEL: &'static str;

    /// Fragment shader path, relative to the assets root.
    const SHADER_PATH: &'static str;

    /// Whether this pass has anything to draw this frame.
    ///
    /// Returning `false` skips the pass entirely rather than paying for a
    /// fullscreen draw that the shader would no-op. Passes that are always
    /// meaningful (the CRT pass itself) can take the default.
    fn is_active(&self) -> bool {
        true
    }
}

/// The GPU-side half of a [`FullscreenEffect`]: everything that can be built
/// once, before the view's texture format is known.
///
/// The pipeline itself is specialized per target format at draw time, because
/// since Bevy 0.19 a view's main texture takes the camera's actual target
/// format rather than a fixed engine-wide default.
#[derive(Resource)]
pub(crate) struct FullscreenPassPipeline<S: FullscreenEffect> {
    layout_desc: BindGroupLayoutDescriptor,
    layout: BindGroupLayout,
    sampler: Sampler,
    vertex: VertexState,
    shader: Handle<Shader>,
    /// Debug labels, built once here rather than per frame: the bind group and
    /// render pass are both rebuilt every frame (see `fullscreen_pass`), and
    /// their label parameters borrow rather than own.
    pipeline_label: String,
    bind_group_label: String,
    pass_label: String,
    _effect: PhantomData<fn() -> S>,
}

impl<S: FullscreenEffect> SpecializedRenderPipeline for FullscreenPassPipeline<S> {
    /// The view's main texture format — the one thing about this pipeline that
    /// isn't known until we have a view to draw into.
    type Key = TextureFormat;

    fn specialize(&self, target_format: TextureFormat) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some(self.pipeline_label.clone().into()),
            layout: vec![self.layout_desc.clone()],
            vertex: self.vertex.clone(),
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                targets: vec![Some(ColorTargetState {
                    format: target_format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        }
    }
}

/// Builds the format-independent half of one pass. Runs in `RenderStartup`,
/// which re-runs after a lost render device, so this rebuilds itself on recovery.
pub(crate) fn init_fullscreen_pipeline<S: FullscreenEffect>(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    fullscreen_shader: Res<FullscreenShader>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout_desc = BindGroupLayoutDescriptor::new(
        format!("{}_bind_group_layout", S::LABEL),
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<S>(true),
            ),
        ),
    );

    commands.insert_resource(FullscreenPassPipeline::<S> {
        layout: pipeline_cache.get_bind_group_layout(&layout_desc),
        layout_desc,
        sampler: render_device.create_sampler(&SamplerDescriptor::default()),
        vertex: fullscreen_shader.to_vertex_state(),
        shader: asset_server.load(S::SHADER_PATH),
        pipeline_label: format!("{}_pipeline", S::LABEL),
        bind_group_label: format!("{}_bind_group", S::LABEL),
        pass_label: format!("{}_pass", S::LABEL),
        _effect: PhantomData,
    });
}

/// Draws one pass of the CRT chain into the current view.
///
/// Each pass consumes the previous one's output: `post_process_write()` hands
/// back the view's current colour texture as `source` and flips the view's main
/// texture to `destination`. That makes chain order load-bearing — see the
/// registration in `crt_effect::plugin`.
pub(crate) fn fullscreen_pass<S: FullscreenEffect>(
    view: ViewQuery<(&ViewTarget, &S, &DynamicUniformIndex<S>)>,
    pass_pipeline: Option<Res<FullscreenPassPipeline<S>>>,
    mut specialized: ResMut<SpecializedRenderPipelines<FullscreenPassPipeline<S>>>,
    pipeline_cache: Res<PipelineCache>,
    settings_uniforms: Res<ComponentUniforms<S>>,
    mut ctx: RenderContext,
) {
    let Some(pass_pipeline) = pass_pipeline else {
        return;
    };

    let (view_target, settings, settings_index) = view.into_inner();

    // Specialize even when the pass has nothing to draw. This call is what
    // queues the pipeline for compilation, so gating it on `is_active` would
    // mean an event-driven pass (lensing, heat, teleport) doesn't start
    // compiling until the frame it first fires — and then misses its own
    // opening frames while the shader builds. Queuing on the first frame
    // instead matches what the pre-0.19 code got from building every pipeline
    // up front in `RenderStartup`.
    let pipeline_id = specialized.specialize(
        &pipeline_cache,
        &pass_pipeline,
        view_target.main_texture_format(),
    );

    // Everything below this point is skipped BEFORE `post_process_write()`, so
    // an inactive pass leaves the view's main texture where the previous pass
    // left it instead of silently swallowing a frame.
    if !settings.is_active() {
        return;
    }

    let Some(pipeline) = pipeline_cache.get_render_pipeline(pipeline_id) else {
        return;
    };

    let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
        return;
    };

    let post_process = view_target.post_process_write();

    let bind_group = ctx.render_device().create_bind_group(
        pass_pipeline.bind_group_label.as_str(),
        &pass_pipeline.layout,
        &BindGroupEntries::sequential((
            post_process.source,
            &pass_pipeline.sampler,
            settings_binding.clone(),
        )),
    );

    let mut render_pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some(pass_pipeline.pass_label.as_str()),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: post_process.destination,
            depth_slice: None,
            resolve_target: None,
            ops: Operations::default(),
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    render_pass.set_render_pipeline(pipeline);
    render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
    render_pass.draw(0..3, 0..1);
}
