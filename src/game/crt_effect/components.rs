use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::ShaderType;

use super::constants::*;

/// Settings component that controls the CRT post-processing effect.
///
/// Attach this to a camera entity to enable the CRT effect on that camera.
/// All parameters are runtime-configurable by mutating this component.
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct CrtEffectSettings {
    /// Screen curvature strength (0.0–0.5).
    pub barrel_distortion: f32,
    /// Scanline darkness (0.0–1.0).
    pub scanline_intensity: f32,
    /// Number of scanlines (100–800).
    pub scanline_count: f32,
    /// RGB subpixel grid visibility (0.0–1.0).
    pub rgb_grid_intensity: f32,
    /// Edge darkening strength (0.0–1.0).
    pub vignette_intensity: f32,
    /// Bright center size before vignette (0.3–1.0).
    pub vignette_radius: f32,
    /// Master on/off toggle (0.0 = off, 1.0 = on).
    pub enabled: f32,
    /// RGB channel separation strength (0.0–0.01).
    pub chromatic_aberration: f32,
    /// Subtle brightness oscillation amount (0.0–0.1).
    pub flicker_intensity: f32,
    /// How much screen corners are rounded off (0.0–0.2).
    pub corner_radius: f32,
    /// Phosphor bloom strength on bright areas (0.0–0.5).
    pub glow_intensity: f32,
    /// Elapsed seconds, updated each frame for time-based effects.
    pub time: f32,
    /// Channel-change effect intensity (0.0 = off, 1.0 = full).
    pub channel_change: f32,
    /// Elapsed time for the channel-change animation.
    pub channel_change_time: f32,
    /// Desaturation intensity (0.0 = full color, 1.0 = fully greyscale).
    pub desaturation: f32,
    /// Viewport X offset in UV space (0.0–1.0). Used for 16:9 letterboxing.
    pub viewport_x: f32,
    /// Viewport Y offset in UV space (0.0–1.0). Used for 16:9 letterboxing.
    pub viewport_y: f32,
    /// Viewport width in UV space (0.0–1.0). Used for 16:9 letterboxing.
    pub viewport_w: f32,
    /// Viewport height in UV space (0.0–1.0). Used for 16:9 letterboxing.
    pub viewport_h: f32,
}

/// Settings component that controls the gravitational lensing post-processing effect.
///
/// Attach this to the same camera entity as `CrtEffectSettings` to enable
/// black hole gravitational lensing. Runs as a separate render pass before CRT.
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct LensingSettings {
    /// Number of active black holes for gravitational lensing (0.0–2.0).
    pub lensing_count: f32,
    /// Global lensing distortion strength.
    pub lensing_strength: f32,
    /// Screen darkening factor during black holes (0.0 = no darkening, 1.0 = full dark).
    pub lensing_darkening: f32,
    /// Black hole 0: screen-space UV X position.
    pub lensing_0_x: f32,
    /// Black hole 0: screen-space UV Y position.
    pub lensing_0_y: f32,
    /// Black hole 0: screen-space influence radius in UV.
    pub lensing_0_radius: f32,
    /// Black hole 1: screen-space UV X position.
    pub lensing_1_x: f32,
    /// Black hole 1: screen-space UV Y position.
    pub lensing_1_y: f32,
    /// Black hole 1: screen-space influence radius in UV.
    pub lensing_1_radius: f32,
}

impl CrtEffectSettings {
    /// Returns true if barrel distortion correction is needed.
    pub fn is_barrel_active(&self) -> bool {
        self.enabled >= 0.5 && self.barrel_distortion != 0.0
    }
}

impl Default for CrtEffectSettings {
    fn default() -> Self {
        Self {
            barrel_distortion: DEFAULT_BARREL_DISTORTION,
            scanline_intensity: DEFAULT_SCANLINE_INTENSITY,
            scanline_count: DEFAULT_SCANLINE_COUNT,
            rgb_grid_intensity: DEFAULT_RGB_GRID_INTENSITY,
            vignette_intensity: DEFAULT_VIGNETTE_INTENSITY,
            vignette_radius: DEFAULT_VIGNETTE_RADIUS,
            enabled: 1.0,
            chromatic_aberration: DEFAULT_CHROMATIC_ABERRATION,
            flicker_intensity: DEFAULT_FLICKER_INTENSITY,
            corner_radius: DEFAULT_CORNER_RADIUS,
            glow_intensity: DEFAULT_GLOW_INTENSITY,
            time: 0.0,
            channel_change: 0.0,
            channel_change_time: 0.0,
            desaturation: 0.0,
            viewport_x: 0.0,
            viewport_y: 0.0,
            viewport_w: 1.0,
            viewport_h: 1.0,
        }
    }
}

impl Default for LensingSettings {
    fn default() -> Self {
        Self {
            lensing_count: 0.0,
            lensing_strength: 0.0,
            lensing_darkening: 0.0,
            lensing_0_x: 0.0,
            lensing_0_y: 0.0,
            lensing_0_radius: 0.0,
            lensing_1_x: 0.0,
            lensing_1_y: 0.0,
            lensing_1_radius: 0.0,
        }
    }
}

/// Timer resource that drives the channel-change animation.
/// Inserted when a `ChannelChangeMessage` is received, removed when finished.
#[derive(Resource)]
pub(crate) struct ChannelChangeTimer {
    pub elapsed: f32,
    pub duration: f32,
}

impl ChannelChangeTimer {
    pub fn new(duration: f32) -> Self {
        Self {
            elapsed: 0.0,
            duration,
        }
    }

    /// Returns the current intensity (0→1→0) using a sine curve.
    pub fn intensity(&self) -> f32 {
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        (t * std::f32::consts::PI).sin()
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// Timer resource that drives the screen desaturation animation.
/// Inserted when a `ScreenDesaturateMessage` is received, removed when finished.
#[derive(Resource)]
pub(crate) struct DesaturationTimer {
    pub elapsed: f32,
    pub duration: f32,
}

impl DesaturationTimer {
    pub fn new(duration: f32) -> Self {
        Self {
            elapsed: 0.0,
            duration,
        }
    }

    /// Returns the current intensity (0→1→0) using a sine curve.
    pub fn intensity(&self) -> f32 {
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        (t * std::f32::consts::PI).sin()
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }
}
