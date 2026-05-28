use bevy::{
    core_pipeline::core_3d::graph::{Core3d, Node3d},
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
        render_resource::*,
        renderer::RenderContext,
        view::ViewTarget,
    },
    ui_render::graph::NodeUi,
};

use super::components::{
    ChannelChangeTimer, ColorblindCorrectionSettings, CrtEffectSettings, DesaturationTimer,
    HeatDistortionSettings, HighContrastSettings, LensingSettings, ScreenFlashTimer,
    TeleportDistortionSettings, VignettePulseTimer,
};
use super::distortion::{
    update_heat_distortion_positions, update_lensing_positions,
    update_teleport_distortion_positions,
};
use super::messages::{
    ChannelChangeMessage, ScreenDesaturateMessage, ScreenFlashMessage, VignettePulseMessage,
};
use super::systems::{
    CorrectedCursorPosition, RawCursorPosition, animate_channel_change, animate_desaturation,
    animate_screen_flash, animate_vignette_pulse, correct_cursor_for_barrel_distortion,
    correct_ui_interaction_for_barrel, handle_channel_change_message, handle_desaturation_message,
    handle_screen_flash_message, handle_vignette_pulse_message,
};
use crate::config::GameConfig;
use crate::game::run_conditions::is_spell_effects_active;

use super::pipeline::{
    ColorblindCorrectionLabel, ColorblindCorrectionNode, CrtEffectPipeline, HeatDistortionNode,
    HighContrastLabel, HighContrastNode, LensingNode, TeleportDistortionLabel,
    TeleportDistortionNode, init_colorblind_pipeline, init_crt_pipeline,
    init_heat_distortion_pipeline, init_high_contrast_pipeline, init_lensing_pipeline,
    init_teleport_distortion_pipeline, sync_colorblind_settings, sync_crt_enabled,
    sync_flicker_intensity, sync_high_contrast, update_crt_time,
};

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
            ExtractComponentPlugin::<TeleportDistortionSettings>::default(),
            UniformComponentPlugin::<TeleportDistortionSettings>::default(),
            ExtractComponentPlugin::<ColorblindCorrectionSettings>::default(),
            UniformComponentPlugin::<ColorblindCorrectionSettings>::default(),
            ExtractComponentPlugin::<HighContrastSettings>::default(),
            UniformComponentPlugin::<HighContrastSettings>::default(),
        ));

        app.init_resource::<RawCursorPosition>();
        app.init_resource::<CorrectedCursorPosition>();
        app.add_message::<ChannelChangeMessage>();
        app.add_message::<ScreenDesaturateMessage>();
        app.add_message::<ScreenFlashMessage>();
        app.add_message::<VignettePulseMessage>();

        app.add_systems(
            Update,
            update_crt_time.run_if(any_with_component::<CrtEffectSettings>),
        );
        // Flash/flicker effects — skipped when reduce_flashes is enabled
        app.add_systems(
            Update,
            handle_channel_change_message.run_if(|config: Res<GameConfig>| !config.reduce_flashes),
        );
        app.add_systems(
            Update,
            animate_channel_change.run_if(resource_exists::<ChannelChangeTimer>),
        );
        app.add_systems(
            Update,
            handle_desaturation_message.run_if(|config: Res<GameConfig>| !config.reduce_flashes),
        );
        app.add_systems(
            Update,
            animate_desaturation.run_if(resource_exists::<DesaturationTimer>),
        );
        app.add_systems(
            Update,
            handle_screen_flash_message.run_if(|config: Res<GameConfig>| !config.reduce_flashes),
        );
        app.add_systems(
            Update,
            animate_screen_flash.run_if(resource_exists::<ScreenFlashTimer>),
        );
        app.add_systems(
            Update,
            handle_vignette_pulse_message.run_if(|config: Res<GameConfig>| !config.reduce_flashes),
        );
        app.add_systems(
            Update,
            animate_vignette_pulse.run_if(resource_exists::<VignettePulseTimer>),
        );
        // Screen-warping distortion — skipped when reduce_motion is enabled.
        // Uses `is_spell_effects_active` so BOTH MP peers run these (the
        // guest must see the same heat shimmer / black-hole lensing /
        // teleport ripple as the host). `is_gameplay_active` would exclude
        // the guest since it gates on `PeerRole::Host`.
        app.add_systems(
            Update,
            (
                update_lensing_positions,
                update_heat_distortion_positions,
                update_teleport_distortion_positions,
            )
                .run_if(is_spell_effects_active)
                .run_if(|config: Res<GameConfig>| !config.reduce_motion),
        );
        app.add_systems(
            Update,
            (
                sync_colorblind_settings,
                sync_crt_enabled,
                sync_flicker_intensity,
                sync_high_contrast,
            )
                .run_if(resource_changed::<GameConfig>),
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
            (
                init_crt_pipeline,
                init_lensing_pipeline,
                init_heat_distortion_pipeline,
                init_teleport_distortion_pipeline,
                init_colorblind_pipeline,
                init_high_contrast_pipeline,
            ),
        );

        render_app
            .add_render_graph_node::<ViewNodeRunner<LensingNode>>(Core3d, LensingLabel)
            .add_render_graph_node::<ViewNodeRunner<TeleportDistortionNode>>(
                Core3d,
                TeleportDistortionLabel,
            )
            .add_render_graph_node::<ViewNodeRunner<HeatDistortionNode>>(
                Core3d,
                HeatDistortionLabel,
            )
            .add_render_graph_node::<ViewNodeRunner<CrtEffectNode>>(Core3d, CrtEffectLabel)
            .add_render_graph_node::<ViewNodeRunner<HighContrastNode>>(Core3d, HighContrastLabel)
            .add_render_graph_node::<ViewNodeRunner<ColorblindCorrectionNode>>(
                Core3d,
                ColorblindCorrectionLabel,
            )
            .add_render_graph_edges(
                Core3d,
                (
                    NodeUi::UiPass,
                    LensingLabel,
                    TeleportDistortionLabel,
                    HeatDistortionLabel,
                    CrtEffectLabel,
                    HighContrastLabel,
                    ColorblindCorrectionLabel,
                    Node3d::Upscaling,
                ),
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
