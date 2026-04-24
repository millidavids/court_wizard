//! Tutorial UI styling constants.

use bevy::prelude::*;

use crate::ui::constants::{BUTTON_BG, BUTTON_BORDER, TEXT_MUTED, TEXT_PRIMARY};

/// Golden glow color for highlighted elements.
pub(super) const GLOW_COLOR: Color = Color::hsla(45.0, 0.95, 0.60, 1.0);

/// Hue used by the pulsing animation in `animate_glow`. Kept in sync with
/// `GLOW_COLOR` so the resting and pulsing colors share a tone.
pub(super) const GLOW_HUE: f32 = 45.0;

/// Glow border width in pixels.
pub(super) const GLOW_BORDER_WIDTH: f32 = 5.0;

/// Overlay background color (nearly transparent — blocks interaction without visible dimming).
pub(super) const OVERLAY_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.01);

/// Tutorial panel background color.
pub(super) const PANEL_BG: Color = Color::hsla(20.0, 0.10, 0.07, 1.0);

/// Tutorial panel border color (golden).
pub(super) const PANEL_BORDER: Color = Color::hsla(42.0, 0.65, 0.45, 1.0);

/// Panel border width in pixels.
pub(super) const PANEL_BORDER_WIDTH: f32 = 2.0;

/// Panel border radius in pixels.
pub(super) const PANEL_BORDER_RADIUS: f32 = 8.0;

/// Panel max width in pixels.
pub(super) const PANEL_MAX_WIDTH: f32 = 500.0;

/// Panel padding in pixels.
pub(super) const PANEL_PADDING: f32 = 24.0;

/// Tutorial text font size.
pub(super) const TEXT_FONT_SIZE: f32 = 11.0;

/// Step counter font size.
pub(super) const STEP_FONT_SIZE: f32 = 9.0;

/// Button font size.
pub(super) const BUTTON_FONT_SIZE: f32 = 10.0;

/// Button width in pixels.
pub(super) const BUTTON_WIDTH: f32 = 120.0;

/// Button height in pixels.
pub(super) const BUTTON_HEIGHT: f32 = 36.0;

/// Button border width in pixels.
pub(super) const BUTTON_BORDER_WIDTH: f32 = 2.0;

/// Next button background color.
pub(super) const NEXT_BUTTON_BG: Color = Color::hsla(270.0, 0.50, 0.30, 0.75);

/// Next button border color.
pub(super) const NEXT_BUTTON_BORDER: Color = Color::hsla(270.0, 0.65, 0.50, 1.0);

/// Skip button background color.
pub(super) const SKIP_BUTTON_BG: Color = BUTTON_BG;

/// Skip button border color.
pub(super) const SKIP_BUTTON_BORDER: Color = BUTTON_BORDER;

/// Text color.
pub(super) const TEXT_COLOR: Color = TEXT_PRIMARY;

/// Muted text color for step counter.
pub(super) const MUTED_TEXT_COLOR: Color = TEXT_MUTED;

/// Glow animation speed (radians per second).
pub(super) const GLOW_ANIMATION_SPEED: f32 = 3.0;


/// GlobalZIndex for the tutorial overlay.
pub(super) const TUTORIAL_Z_INDEX: i32 = 1000;

/// Margin from screen edges for panel positioning.
pub(super) const PANEL_MARGIN: f32 = 40.0;
