//! Components for the CRT post-processing pipeline.
//!
//! Split into three concerns:
//! - `effect_settings`: GPU-side uniform structs for visual effects (CRT, lensing, heat, teleport distortion)
//! - `accessibility_settings`: GPU-side uniform structs for accessibility (colorblind correction, high contrast)
//! - `timers`: CPU-side animation timer resources (`ChannelChangeTimer`, etc.)

mod accessibility_settings;
mod effect_settings;
mod timers;

pub use accessibility_settings::ColorblindCorrectionSettings;
pub use accessibility_settings::HighContrastSettings;
pub use effect_settings::CrtEffectSettings;
pub use effect_settings::HeatDistortionSettings;
pub use effect_settings::LensingSettings;
pub use effect_settings::TeleportDistortionSettings;

pub(crate) use timers::ChannelChangeTimer;
pub(crate) use timers::DesaturationTimer;
pub(crate) use timers::ScreenFlashTimer;
pub(crate) use timers::VignettePulseTimer;
