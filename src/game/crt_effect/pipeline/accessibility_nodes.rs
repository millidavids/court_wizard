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
    ColorblindCorrectionSettings, CrtEffectSettings, HighContrastSettings,
};
use super::accessibility_pipeline::{ColorblindCorrectionPipeline, HighContrastPipeline};
use crate::config::GameConfig;

// --- Colorblind Correction render node ---

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct ColorblindCorrectionLabel;

#[derive(Default)]
pub(crate) struct ColorblindCorrectionNode;

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

// --- High Contrast render node ---

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct HighContrastLabel;

#[derive(Default)]
pub(crate) struct HighContrastNode;

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

// --- Main-world sync / update systems ---
//
// The `sync_*` systems below only run when `GameConfig` changes, and they assume
// the single camera spawned in `main::setup` at `Startup` — which already exists
// by the first `Update`, so the saved config lands before the first frame is drawn.
// A camera spawned any later would keep its `Default` settings until the next
// config change, so anything that spawns one needs its own sync.
//
// They deliberately write unconditionally rather than caching the previous value:
// these settings components are extracted and re-uploaded to the GPU every frame
// regardless (`extract_components` has no `Changed` filter), so a cache saves
// nothing — and a cache seeded with a type default silently skips the very first
// sync whenever that default happens to match the saved config.

pub(crate) fn update_crt_time(time: Res<Time>, mut query: Query<&mut CrtEffectSettings>) {
    for mut settings in &mut query {
        settings.time = time.elapsed_secs();
    }
}

/// Syncs GameConfig colorblind settings to the camera's ColorblindCorrectionSettings component.
pub(crate) fn sync_colorblind_settings(
    config: Res<GameConfig>,
    mut query: Query<&mut ColorblindCorrectionSettings>,
) {
    let new_settings =
        ColorblindCorrectionSettings::for_type(config.colorblind_type, config.colorblind_strength);
    for mut settings in &mut query {
        *settings = new_settings;
    }
}

/// Syncs the CRT effect enabled state from GameConfig to the camera component.
/// Also zeroes barrel_distortion when disabled so the shader samples undistorted UVs
/// and cursor correction (which checks `is_barrel_active()`) is skipped.
pub(crate) fn sync_crt_enabled(config: Res<GameConfig>, mut query: Query<&mut CrtEffectSettings>) {
    for mut settings in &mut query {
        if config.crt_enabled {
            settings.enabled = 1.0;
            settings.barrel_distortion = super::super::constants::DEFAULT_BARREL_DISTORTION;
        } else {
            settings.enabled = 0.0;
            settings.barrel_distortion = 0.0;
        }
    }
}

/// Syncs high contrast settings from GameConfig to the camera component.
pub(crate) fn sync_high_contrast(
    config: Res<GameConfig>,
    mut query: Query<&mut HighContrastSettings>,
) {
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
pub(crate) fn sync_flicker_intensity(
    config: Res<GameConfig>,
    mut query: Query<&mut CrtEffectSettings>,
) {
    let intensity = if config.reduce_flashes {
        0.0
    } else {
        super::super::constants::DEFAULT_FLICKER_INTENSITY
    };
    for mut settings in &mut query {
        settings.flicker_intensity = intensity;
    }
}
