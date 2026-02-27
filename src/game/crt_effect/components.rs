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
    /// Padding for 16-byte GPU alignment.
    pub _padding1: f32,
    /// Padding for 16-byte GPU alignment.
    pub _padding2: f32,
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
            _padding1: 0.0,
            _padding2: 0.0,
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
