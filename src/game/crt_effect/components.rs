use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::ShaderType;

use crate::config::ColorblindType;
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
    /// Screen flash red channel (0.0–1.0).
    pub screen_flash_r: f32,
    /// Screen flash green channel (0.0–1.0).
    pub screen_flash_g: f32,
    /// Screen flash blue channel (0.0–1.0).
    pub screen_flash_b: f32,
    /// Screen flash intensity (0.0 = off, 1.0 = full). Drives the blend strength.
    pub screen_flash_intensity: f32,
    /// Vignette pulse: temporary extra darkening at edges (0.0–1.0).
    pub vignette_pulse: f32,
}

/// Settings component that controls the gravitational lensing post-processing effect.
///
/// Attach this to the same camera entity as `CrtEffectSettings` to enable
/// black hole gravitational lensing. Runs as a separate render pass before CRT.
/// Supports up to 2 black holes (slots 0-1, with center darkening) plus
/// 2 rift endpoints (slots 2-3, lensing only, no darkening).
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct LensingSettings {
    /// Total number of active lensing sources (0.0–4.0).
    pub lensing_count: f32,
    /// Global lensing distortion strength.
    pub lensing_strength: f32,
    /// Screen darkening factor during black holes (0.0 = no darkening, 1.0 = full dark).
    pub lensing_darkening: f32,
    /// Slot 0 (black hole): screen-space UV X position.
    pub lensing_0_x: f32,
    /// Slot 0 (black hole): screen-space UV Y position.
    pub lensing_0_y: f32,
    /// Slot 0 (black hole): screen-space influence radius in UV.
    pub lensing_0_radius: f32,
    /// Slot 1 (black hole): screen-space UV X position.
    pub lensing_1_x: f32,
    /// Slot 1 (black hole): screen-space UV Y position.
    pub lensing_1_y: f32,
    /// Slot 1 (black hole): screen-space influence radius in UV.
    pub lensing_1_radius: f32,
    /// Slot 2 (rift): screen-space UV X position.
    pub lensing_2_x: f32,
    /// Slot 2 (rift): screen-space UV Y position.
    pub lensing_2_y: f32,
    /// Slot 2 (rift): screen-space influence radius in UV.
    pub lensing_2_radius: f32,
    /// Slot 3 (rift): screen-space UV X position.
    pub lensing_3_x: f32,
    /// Slot 3 (rift): screen-space UV Y position.
    pub lensing_3_y: f32,
    /// Slot 3 (rift): screen-space influence radius in UV.
    pub lensing_3_radius: f32,
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
            screen_flash_r: 0.0,
            screen_flash_g: 0.0,
            screen_flash_b: 0.0,
            screen_flash_intensity: 0.0,
            vignette_pulse: 0.0,
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
            lensing_2_x: 0.0,
            lensing_2_y: 0.0,
            lensing_2_radius: 0.0,
            lensing_3_x: 0.0,
            lensing_3_y: 0.0,
            lensing_3_radius: 0.0,
        }
    }
}

/// Settings component that controls the heat distortion post-processing effect.
///
/// Attach this to the same camera entity as `CrtEffectSettings` to enable
/// heat shimmer around wall of fire effects. Supports up to 4 walls.
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct HeatDistortionSettings {
    /// Number of active walls (0.0–4.0).
    pub count: f32,
    /// Global distortion strength.
    pub strength: f32,
    /// Elapsed time for animation.
    pub time: f32,
    pub _pad0: f32,
    // Wall 0 endpoints in screen UV
    pub wall_0_start_x: f32,
    pub wall_0_start_y: f32,
    pub wall_0_end_x: f32,
    pub wall_0_end_y: f32,
    // Wall 1 endpoints in screen UV
    pub wall_1_start_x: f32,
    pub wall_1_start_y: f32,
    pub wall_1_end_x: f32,
    pub wall_1_end_y: f32,
    // Wall 2 endpoints in screen UV
    pub wall_2_start_x: f32,
    pub wall_2_start_y: f32,
    pub wall_2_end_x: f32,
    pub wall_2_end_y: f32,
    // Wall 3 endpoints in screen UV
    pub wall_3_start_x: f32,
    pub wall_3_start_y: f32,
    pub wall_3_end_x: f32,
    pub wall_3_end_y: f32,
    // Influence radius per wall (in UV space)
    pub wall_0_radius: f32,
    pub wall_1_radius: f32,
    pub wall_2_radius: f32,
    pub wall_3_radius: f32,
}

impl Default for HeatDistortionSettings {
    fn default() -> Self {
        Self {
            count: 0.0,
            strength: 0.001,
            time: 0.0,
            _pad0: 0.0,
            wall_0_start_x: 0.0,
            wall_0_start_y: 0.0,
            wall_0_end_x: 0.0,
            wall_0_end_y: 0.0,
            wall_1_start_x: 0.0,
            wall_1_start_y: 0.0,
            wall_1_end_x: 0.0,
            wall_1_end_y: 0.0,
            wall_2_start_x: 0.0,
            wall_2_start_y: 0.0,
            wall_2_end_x: 0.0,
            wall_2_end_y: 0.0,
            wall_3_start_x: 0.0,
            wall_3_start_y: 0.0,
            wall_3_end_x: 0.0,
            wall_3_end_y: 0.0,
            wall_0_radius: 0.0,
            wall_1_radius: 0.0,
            wall_2_radius: 0.0,
            wall_3_radius: 0.0,
        }
    }
}

/// Settings component that controls the teleport spatial distortion post-processing effect.
///
/// Attach this to the same camera entity as `CrtEffectSettings` to enable
/// radial ripple effects at teleport source/destination points. Supports up to 4 points.
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct TeleportDistortionSettings {
    /// Number of active distortion points (0.0–4.0).
    pub count: f32,
    /// Elapsed time for wave animation.
    pub time: f32,
    /// Global distortion strength multiplier.
    pub strength: f32,
    /// Wave frequency (number of concentric rings).
    pub frequency: f32,
    /// Wave propagation speed.
    pub speed: f32,
    pub _pad0: f32,
    pub _pad1: f32,
    pub _pad2: f32,
    // Point 0
    pub point_0_x: f32,
    pub point_0_y: f32,
    pub point_0_radius: f32,
    pub point_0_intensity: f32,
    // Point 1
    pub point_1_x: f32,
    pub point_1_y: f32,
    pub point_1_radius: f32,
    pub point_1_intensity: f32,
    // Point 2
    pub point_2_x: f32,
    pub point_2_y: f32,
    pub point_2_radius: f32,
    pub point_2_intensity: f32,
    // Point 3
    pub point_3_x: f32,
    pub point_3_y: f32,
    pub point_3_radius: f32,
    pub point_3_intensity: f32,
}

impl Default for TeleportDistortionSettings {
    fn default() -> Self {
        use crate::game::units::wizard::spells::teleport::vfx_constants;
        Self {
            count: 0.0,
            time: 0.0,
            strength: vfx_constants::RIPPLE_STRENGTH,
            frequency: vfx_constants::RIPPLE_FREQUENCY,
            speed: vfx_constants::RIPPLE_SPEED,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            point_0_x: 0.0,
            point_0_y: 0.0,
            point_0_radius: 0.0,
            point_0_intensity: 0.0,
            point_1_x: 0.0,
            point_1_y: 0.0,
            point_1_radius: 0.0,
            point_1_intensity: 0.0,
            point_2_x: 0.0,
            point_2_y: 0.0,
            point_2_radius: 0.0,
            point_2_intensity: 0.0,
            point_3_x: 0.0,
            point_3_y: 0.0,
            point_3_radius: 0.0,
            point_3_intensity: 0.0,
        }
    }
}

impl TeleportDistortionSettings {
    /// Sets a distortion point by index (0–3), avoiding repetitive match arms.
    pub fn set_point(&mut self, index: u32, x: f32, y: f32, radius: f32, intensity: f32) {
        match index {
            0 => {
                self.point_0_x = x;
                self.point_0_y = y;
                self.point_0_radius = radius;
                self.point_0_intensity = intensity;
            }
            1 => {
                self.point_1_x = x;
                self.point_1_y = y;
                self.point_1_radius = radius;
                self.point_1_intensity = intensity;
            }
            2 => {
                self.point_2_x = x;
                self.point_2_y = y;
                self.point_2_radius = radius;
                self.point_2_intensity = intensity;
            }
            3 => {
                self.point_3_x = x;
                self.point_3_y = y;
                self.point_3_radius = radius;
                self.point_3_intensity = intensity;
            }
            _ => {}
        }
    }
}

/// Settings component that controls the colorblind correction post-processing effect.
///
/// Attach this to the same camera entity as `CrtEffectSettings` to enable
/// daltonization-based color correction. Runs as a separate render pass after CRT.
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct ColorblindCorrectionSettings {
    /// Simulation matrix row 0 (xyz used, w padding).
    pub sim_row0: Vec4,
    /// Simulation matrix row 1 (xyz used, w padding).
    pub sim_row1: Vec4,
    /// Simulation matrix row 2 (xyz used, w padding).
    pub sim_row2: Vec4,
    /// Error redistribution matrix row 0 (xyz used, w padding).
    pub err_row0: Vec4,
    /// Error redistribution matrix row 1 (xyz used, w padding).
    pub err_row1: Vec4,
    /// Error redistribution matrix row 2 (xyz used, w padding).
    pub err_row2: Vec4,
    /// Correction strength (0.0 = no correction, 1.0 = full).
    pub strength: f32,
    /// Master toggle (0.0 = off, 1.0 = on).
    pub enabled: f32,
    pub _pad0: f32,
    pub _pad1: f32,
}

impl Default for ColorblindCorrectionSettings {
    fn default() -> Self {
        Self {
            sim_row0: Vec4::new(1.0, 0.0, 0.0, 0.0),
            sim_row1: Vec4::new(0.0, 1.0, 0.0, 0.0),
            sim_row2: Vec4::new(0.0, 0.0, 1.0, 0.0),
            err_row0: Vec4::ZERO,
            err_row1: Vec4::ZERO,
            err_row2: Vec4::ZERO,
            strength: 1.0,
            enabled: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
        }
    }
}

impl ColorblindCorrectionSettings {
    /// Creates settings for a given CVD type and strength.
    pub fn for_type(cvd_type: ColorblindType, strength: f32) -> Self {
        let (sim, err, enabled) = match cvd_type {
            ColorblindType::None => return Self::default(),
            ColorblindType::Protanopia => (PROTAN_SIM, PROTAN_DEUTAN_ERR, 1.0),
            ColorblindType::Deuteranopia => (DEUTAN_SIM, PROTAN_DEUTAN_ERR, 1.0),
            ColorblindType::Tritanopia => (TRITAN_SIM, TRITAN_ERR, 1.0),
        };
        Self {
            sim_row0: Vec4::new(sim[0], sim[1], sim[2], 0.0),
            sim_row1: Vec4::new(sim[3], sim[4], sim[5], 0.0),
            sim_row2: Vec4::new(sim[6], sim[7], sim[8], 0.0),
            err_row0: Vec4::new(err[0], err[1], err[2], 0.0),
            err_row1: Vec4::new(err[3], err[4], err[5], 0.0),
            err_row2: Vec4::new(err[6], err[7], err[8], 0.0),
            strength,
            enabled,
            _pad0: 0.0,
            _pad1: 0.0,
        }
    }
}

/// Settings component that controls the high contrast post-processing effect.
///
/// Attach this to the same camera entity as `CrtEffectSettings` to enable
/// sigmoidal contrast + saturation boost. Runs as a separate render pass after CRT.
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub struct HighContrastSettings {
    /// Effect strength (0.0 = off, 1.0 = full).
    pub strength: f32,
    /// Master toggle (0.0 = off, 1.0 = on).
    pub enabled: f32,
    pub _pad0: f32,
    pub _pad1: f32,
}

impl Default for HighContrastSettings {
    fn default() -> Self {
        Self {
            strength: 0.0,
            enabled: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
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

/// Timer resource that drives the screen flash animation.
/// Inserted when a `ScreenFlashMessage` is received, removed when finished.
#[derive(Resource)]
pub(crate) struct ScreenFlashTimer {
    pub elapsed: f32,
    pub duration: f32,
    pub color: [f32; 3],
    pub peak_intensity: f32,
}

impl ScreenFlashTimer {
    pub fn new(color: [f32; 3], duration: f32, peak_intensity: f32) -> Self {
        Self {
            elapsed: 0.0,
            duration,
            color,
            peak_intensity,
        }
    }

    /// Returns the current intensity (0→peak→0) using a sine curve.
    pub fn intensity(&self) -> f32 {
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        (t * std::f32::consts::PI).sin() * self.peak_intensity
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// Timer resource that drives the vignette pulse animation.
#[derive(Resource)]
pub(crate) struct VignettePulseTimer {
    pub elapsed: f32,
    pub duration: f32,
    pub peak_intensity: f32,
}

impl VignettePulseTimer {
    pub fn new(duration: f32, peak_intensity: f32) -> Self {
        Self {
            elapsed: 0.0,
            duration,
            peak_intensity,
        }
    }

    /// Returns the current intensity (0→peak→0) using a sine curve.
    pub fn intensity(&self) -> f32 {
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        (t * std::f32::consts::PI).sin() * self.peak_intensity
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
