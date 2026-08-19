//! The six passes of the CRT chain, in draw order.
//!
//! Each is a shader path plus the test for "is this pass doing anything right
//! now"; all the pipeline and render-pass plumbing lives in `fullscreen.rs`.

use super::super::components::{
    ColorblindCorrectionSettings, CrtEffectSettings, HeatDistortionSettings, HighContrastSettings,
    LensingSettings, TeleportDistortionSettings,
};
use super::fullscreen::FullscreenEffect;

impl FullscreenEffect for LensingSettings {
    const LABEL: &'static str = "lensing";
    const SHADER_PATH: &'static str = "shaders/gravitational_lensing.wgsl";

    fn is_active(&self) -> bool {
        self.lensing_count >= 0.5
    }
}

impl FullscreenEffect for TeleportDistortionSettings {
    const LABEL: &'static str = "teleport_distortion";
    const SHADER_PATH: &'static str = "shaders/teleport_distortion.wgsl";

    fn is_active(&self) -> bool {
        self.count >= 0.5
    }
}

impl FullscreenEffect for HeatDistortionSettings {
    const LABEL: &'static str = "heat_distortion";
    const SHADER_PATH: &'static str = "shaders/heat_distortion.wgsl";

    fn is_active(&self) -> bool {
        self.count >= 0.5
    }
}

/// The CRT pass proper. Always drawn — its `enabled` uniform is read by the
/// shader, which still has scanline, flicker and channel-change work to do with
/// curvature switched off.
impl FullscreenEffect for CrtEffectSettings {
    const LABEL: &'static str = "crt_effect";
    const SHADER_PATH: &'static str = "shaders/crt_effect.wgsl";
}

impl FullscreenEffect for HighContrastSettings {
    const LABEL: &'static str = "high_contrast";
    const SHADER_PATH: &'static str = "shaders/high_contrast.wgsl";

    fn is_active(&self) -> bool {
        self.enabled >= 0.5
    }
}

impl FullscreenEffect for ColorblindCorrectionSettings {
    const LABEL: &'static str = "colorblind_correction";
    const SHADER_PATH: &'static str = "shaders/colorblind_correction.wgsl";

    fn is_active(&self) -> bool {
        self.enabled >= 0.5
    }
}
