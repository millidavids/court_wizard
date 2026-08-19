//! Pushes `GameConfig` and the clock into the camera's CRT settings components.
//!
//! The `sync_*` systems below only run when `GameConfig` changes, and they assume
//! the single camera spawned in `main::setup` at `Startup` — which already exists
//! by the first `Update`, so the saved config lands before the first frame is drawn.
//! A camera spawned any later would keep its `Default` settings until the next
//! config change, so anything that spawns one needs its own sync.
//!
//! They deliberately write unconditionally rather than caching the previous value:
//! these settings components are extracted and re-uploaded to the GPU every frame
//! regardless (`extract_components` has no `Changed` filter), so a cache saves
//! nothing — and a cache seeded with a type default silently skips the very first
//! sync whenever that default happens to match the saved config.

use bevy::prelude::*;

use super::components::{ColorblindCorrectionSettings, CrtEffectSettings, HighContrastSettings};
use super::constants::{DEFAULT_BARREL_DISTORTION, DEFAULT_FLICKER_INTENSITY};
use crate::config::GameConfig;

pub(super) fn update_crt_time(time: Res<Time>, mut query: Query<&mut CrtEffectSettings>) {
    for mut settings in &mut query {
        settings.time = time.elapsed_secs();
    }
}

/// Syncs GameConfig colorblind settings to the camera's ColorblindCorrectionSettings component.
pub(super) fn sync_colorblind_settings(
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
pub(super) fn sync_crt_enabled(config: Res<GameConfig>, mut query: Query<&mut CrtEffectSettings>) {
    for mut settings in &mut query {
        if config.crt_enabled {
            settings.enabled = 1.0;
            settings.barrel_distortion = DEFAULT_BARREL_DISTORTION;
        } else {
            settings.enabled = 0.0;
            settings.barrel_distortion = 0.0;
        }
    }
}

/// Syncs high contrast settings from GameConfig to the camera component.
pub(super) fn sync_high_contrast(
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
pub(super) fn sync_flicker_intensity(
    config: Res<GameConfig>,
    mut query: Query<&mut CrtEffectSettings>,
) {
    let intensity = if config.reduce_flashes {
        0.0
    } else {
        DEFAULT_FLICKER_INTENSITY
    };
    for mut settings in &mut query {
        settings.flicker_intensity = intensity;
    }
}
