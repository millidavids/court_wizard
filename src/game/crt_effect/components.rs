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
        }
    }
}
