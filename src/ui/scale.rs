//! UI scale system — keeps global UiScale in sync with the CRT viewport width.

use bevy::prelude::*;
use bevy::ui::UiScale as BevyUiScale;
use bevy::window::PrimaryWindow;

/// Updates the global UI scale based on the 16:9 viewport width.
///
/// Uses Bevy's built-in UiScale resource to scale all UI elements.
/// Calculates scale factor relative to a base width of 1920px, then applies
/// a 1.5x multiplier to make everything larger.
/// Uses the CRT viewport width (which enforces 16:9) rather than the
/// full window width, so UI scales correctly with letterboxing/pillarboxing.
pub(super) fn update_ui_scale(
    mut ui_scale: ResMut<BevyUiScale>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    crt_query: Query<&crate::game::crt_effect::CrtEffectSettings>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };

    // Use viewport width from CRT settings (fraction of window)
    let viewport_fraction = if let Ok(settings) = crt_query.single() {
        settings.viewport_w
    } else {
        1.0
    };
    let logical_width = window.width() * viewport_fraction;

    const BASE_WIDTH: f32 = 1920.0;
    const SCALE_MULTIPLIER: f32 = 1.5;
    let new_scale = (logical_width / BASE_WIDTH) * SCALE_MULTIPLIER;

    if (ui_scale.0 - new_scale).abs() > 0.001 {
        ui_scale.0 = new_scale;
    }
}
