//! Tutorial UI styling constants.

use bevy::prelude::*;

/// Golden glow color for highlighted elements.
pub(super) const GLOW_COLOR: Color = Color::hsla(45.0, 1.0, 0.65, 1.0);

/// Glow border width in pixels.
pub(super) const GLOW_BORDER_WIDTH: f32 = 5.0;

/// Overlay background color (nearly transparent — blocks interaction without visible dimming).
pub(super) const OVERLAY_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.01);

/// Tutorial panel background color.
pub(super) const PANEL_BG: Color = Color::hsla(220.0, 0.08, 0.08, 1.0);

/// Tutorial panel border color (golden).
pub(super) const PANEL_BORDER: Color = Color::hsla(45.0, 0.8, 0.5, 1.0);

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
pub(super) const NEXT_BUTTON_BG: Color = Color::hsla(45.0, 0.7, 0.35, 1.0);

/// Next button border color.
pub(super) const NEXT_BUTTON_BORDER: Color = Color::hsla(45.0, 0.8, 0.5, 1.0);

/// Skip button background color.
pub(super) const SKIP_BUTTON_BG: Color = Color::hsla(0.0, 0.0, 0.15, 1.0);

/// Skip button border color.
pub(super) const SKIP_BUTTON_BORDER: Color = Color::hsla(0.0, 0.0, 0.4, 1.0);

/// Text color.
pub(super) const TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.9, 1.0);

/// Muted text color for step counter.
pub(super) const MUTED_TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.6, 1.0);

/// Glow animation speed (radians per second).
pub(super) const GLOW_ANIMATION_SPEED: f32 = 3.0;

/// GlobalZIndex for the tutorial overlay.
pub(super) const TUTORIAL_Z_INDEX: i32 = 1000;

/// Margin from screen edges for panel positioning.
pub(super) const PANEL_MARGIN: f32 = 40.0;
