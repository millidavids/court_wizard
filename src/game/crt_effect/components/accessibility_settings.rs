use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::ShaderType;

use crate::config::ColorblindType;
use crate::game::crt_effect::constants::*;

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
