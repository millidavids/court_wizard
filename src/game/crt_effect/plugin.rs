use bevy::{
    core_pipeline::schedule::Core3d,
    prelude::*,
    render::{
        GpuResourceAppExt, RenderApp, RenderStartup,
        extract_component::{ExtractComponentPlugin, UniformComponentPlugin},
        render_resource::SpecializedRenderPipelines,
    },
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

use super::config_sync::{
    sync_colorblind_settings, sync_crt_enabled, sync_flicker_intensity, sync_high_contrast,
    update_crt_time,
};
use super::pipeline::{FullscreenPassPipeline, fullscreen_pass, init_fullscreen_pipeline};

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

        render_app
            .init_gpu_resource::<SpecializedRenderPipelines<FullscreenPassPipeline<LensingSettings>>>()
            .init_gpu_resource::<SpecializedRenderPipelines<FullscreenPassPipeline<TeleportDistortionSettings>>>()
            .init_gpu_resource::<SpecializedRenderPipelines<FullscreenPassPipeline<HeatDistortionSettings>>>()
            .init_gpu_resource::<SpecializedRenderPipelines<FullscreenPassPipeline<CrtEffectSettings>>>()
            .init_gpu_resource::<SpecializedRenderPipelines<FullscreenPassPipeline<HighContrastSettings>>>()
            .init_gpu_resource::<SpecializedRenderPipelines<FullscreenPassPipeline<ColorblindCorrectionSettings>>>();

        render_app.add_systems(
            RenderStartup,
            (
                init_fullscreen_pipeline::<LensingSettings>,
                init_fullscreen_pipeline::<TeleportDistortionSettings>,
                init_fullscreen_pipeline::<HeatDistortionSettings>,
                init_fullscreen_pipeline::<CrtEffectSettings>,
                init_fullscreen_pipeline::<HighContrastSettings>,
                init_fullscreen_pipeline::<ColorblindCorrectionSettings>,
            ),
        );

        // The chain runs AFTER the UI pass and before upscaling, so the UI is
        // warped and tinted along with the scene rather than pasted flat on top
        // — that is the whole look, and `correct_ui_interaction_for_barrel`
        // above exists to undo the barrel warp for cursor hit-testing.
        //
        // Note this is deliberately NOT `Core3dSystems::PostProcess`: bevy
        // registers `ui_pass` as `.after(Core3dSystems::PostProcess)`, so
        // joining that set would put us underneath the UI.
        //
        // `.chain()` is load-bearing twice over: each pass consumes the previous
        // one's `post_process_write()` output, and render systems flush their
        // command buffers in topological order, so system order is submission
        // order.
        render_app.add_systems(
            Core3d,
            (
                fullscreen_pass::<LensingSettings>,
                fullscreen_pass::<TeleportDistortionSettings>,
                fullscreen_pass::<HeatDistortionSettings>,
                fullscreen_pass::<CrtEffectSettings>,
                fullscreen_pass::<HighContrastSettings>,
                fullscreen_pass::<ColorblindCorrectionSettings>,
            )
                .chain()
                .after(bevy::ui_render::ui_pass)
                .before(bevy::core_pipeline::upscaling::upscaling),
        );
    }
}
