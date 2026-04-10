//! Global UI color palette and shared styling constants.
//!
//! Single source of truth for colors used across all UI modules.
//! Module-specific colors (element colors, team colors, bar fills, etc.)
//! remain in their respective module constants files.

use bevy::prelude::*;

// ── Text Hierarchy ──────────────────────────────────────────────────────────

/// Primary text color for titles, headings, and important text (warm parchment white).
pub const TEXT_PRIMARY: Color = Color::hsla(40.0, 0.15, 0.90, 1.0);

/// Body text color for general content and descriptions (warm off-white).
pub const TEXT_BODY: Color = Color::hsla(40.0, 0.10, 0.82, 1.0);

/// Muted text color for labels, subtitles, and secondary information.
pub const TEXT_MUTED: Color = Color::hsla(35.0, 0.08, 0.62, 1.0);

/// Disabled text color for locked or unavailable items.
pub const TEXT_DISABLED: Color = Color::hsla(30.0, 0.06, 0.55, 1.0);

/// Placeholder text color for empty slots and invisible hints.
pub const TEXT_PLACEHOLDER: Color = Color::hsla(30.0, 0.05, 0.45, 1.0);

// ── Semantic Accents ────────────────────────────────────────────────────────

/// Arcane insight / information highlight color (light blue).
pub const INSIGHT_COLOR: Color = Color::srgb(0.6, 0.8, 1.0);

/// Purple accent for active/selected states and highlights.
pub const GOLD_ACCENT: Color = Color::hsla(270.0, 0.65, 0.55, 1.0);

/// Success state color (green).
pub const SUCCESS_COLOR: Color = Color::hsla(120.0, 0.6, 0.5, 1.0);

/// Error / danger state color (red).
pub const ERROR_COLOR: Color = Color::hsla(0.0, 0.6, 0.5, 1.0);

/// Warning / pending state color (gold).
pub const WARNING_COLOR: Color = Color::hsla(45.0, 0.6, 0.5, 1.0);

// ── Overlay / Container ─────────────────────────────────────────────────────

/// Transparent overlay behind content (the content box provides visual separation).
pub const OVERLAY_BG: Color = Color::NONE;

/// Translucent background for page content containers (warm charcoal).
pub const CONTENT_BG: Color = Color::hsla(20.0, 0.12, 0.07, 0.65);

/// Subtle border for page content containers (purple-tinted).
pub const CONTENT_BORDER: Color = Color::hsla(270.0, 0.25, 0.22, 0.5);

/// Background for inner scrollable areas within page containers.
pub const SCROLL_BG: Color = Color::hsla(20.0, 0.08, 0.04, 0.35);

/// Border for inner scrollable areas.
pub const SCROLL_BORDER: Color = Color::hsla(270.0, 0.15, 0.22, 0.5);

/// Shadow color for elevated UI panels (warm-tinted).
pub const SHADOW_COLOR: Color = Color::hsla(25.0, 0.30, 0.05, 0.6);

/// Shadow color for inner scroll areas (slightly lighter, warm).
pub const SCROLL_SHADOW_COLOR: Color = Color::hsla(25.0, 0.20, 0.05, 0.4);

/// Drop shadow color behind text.
pub const TEXT_SHADOW_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.5);

// ── Buttons (warm brown / purple trim) ──────────────────────────────────────

/// Standard button background color (warm charcoal).
pub const BUTTON_BG: Color = Color::hsla(25.0, 0.18, 0.14, 0.85);

/// Standard button border color (vibrant purple trim).
pub const BUTTON_BORDER: Color = Color::hsla(270.0, 0.60, 0.45, 0.8);

/// Subtle/muted button background color (darker warm brown).
pub const BUTTON_BG_SUBTLE: Color = Color::hsla(25.0, 0.15, 0.07, 0.6);

/// Subtle/muted button border color (muted purple).
pub const BUTTON_BORDER_SUBTLE: Color = Color::hsla(270.0, 0.45, 0.35, 0.6);

// ── Detail Panels ───────────────────────────────────────────────────────────

/// Detail panel background (warm dark, semi-transparent).
pub const DETAIL_BG: Color = Color::hsla(20.0, 0.10, 0.08, 0.80);

/// Detail panel border (purple).
pub const DETAIL_BORDER: Color = Color::hsla(270.0, 0.45, 0.38, 0.9);

/// List area background (deepest warm, semi-transparent).
pub const LIST_BG: Color = Color::hsla(20.0, 0.08, 0.06, 0.80);

/// List area border (warm dark bronze).
pub const LIST_BORDER: Color = Color::hsla(30.0, 0.15, 0.18, 0.8);

// ── Ornate Frame ────────────────────────────────────────────────────────

/// Outline color for ornate panel frames and button edges at rest (purple).
pub const FRAME_OUTLINE_COLOR: Color = Color::hsla(270.0, 0.50, 0.40, 0.7);

/// Outline width for panel frames.
pub const FRAME_OUTLINE_WIDTH: f32 = 1.0;

/// Gap between border and outline for ornate frames.
pub const FRAME_OUTLINE_OFFSET: f32 = 2.0;

/// Outer ring color for ornate panel frames (dark purple).
pub const FRAME_OUTER_RING_COLOR: Color = Color::hsla(270.0, 0.25, 0.15, 0.5);

/// Outer ring width (BoxShadow spread with zero blur).
pub const FRAME_OUTER_RING_WIDTH: f32 = 3.0;

/// Base spread for page container shadows (accounts for outline + outer ring).
pub const FRAME_SHADOW_SPREAD_BASE: f32 = FRAME_OUTER_RING_WIDTH + FRAME_OUTLINE_WIDTH + FRAME_OUTLINE_OFFSET;

// ── Button Interaction States ───────────────────────────────────────────

/// Button shadow color (warm dark, shared by normal and pressed states).
pub const BUTTON_SHADOW_COLOR: Color = Color::hsla(25.0, 0.20, 0.05, 0.3);

/// Outline color when button is pressed (brilliant purple, brighter than hover).
pub const BUTTON_PRESSED_OUTLINE: Color = Color::hsla(270.0, 0.85, 0.65, 1.0);

/// Outline color when button is hovered (vibrant purple glow).
pub const BUTTON_HOVERED_OUTLINE: Color = Color::hsla(270.0, 0.70, 0.55, 0.85);

/// Inner glow color on button hover.
pub const BUTTON_GLOW_INNER: Color = Color::hsla(270.0, 0.65, 0.50, 0.30);

/// Outer glow color on button hover.
pub const BUTTON_GLOW_OUTER: Color = Color::hsla(270.0, 0.55, 0.40, 0.15);

/// Inner glow color on button press (brighter than hover).
pub const BUTTON_PRESS_GLOW_INNER: Color = Color::hsla(270.0, 0.60, 0.50, 0.35);

/// Outer glow color on button press.
pub const BUTTON_PRESS_GLOW_OUTER: Color = Color::hsla(270.0, 0.50, 0.40, 0.15);

// ── 3D Button Depth ────────────────────────────────────────────────────

/// How far the front face sits above the edge at rest (pixels).
pub const BUTTON_3D_OFFSET_REST: f32 = -4.0;

/// How far the front face rises on hover (pixels).
pub const BUTTON_3D_OFFSET_HOVER: f32 = -6.0;

/// How far the front face drops on press (pixels, closer to 0 = more edge covered).
pub const BUTTON_3D_OFFSET_PRESSED: f32 = 0.0;

/// Animation speed for the 3D button lerp (higher = snappier).
pub const BUTTON_3D_ANIM_SPEED: f32 = 16.0;

/// Edge color darkening factor (multiplied against button background lightness).
pub const BUTTON_EDGE_DARKEN: f32 = 0.55;

// ── Two-Panel Layout ─────────────────────────────────────────────────────

/// Gap between left and right panels.
pub const TWO_PANEL_GAP: f32 = 30.0;

/// Width of the left detail panel.
pub const LEFT_PANEL_WIDTH: f32 = 300.0;

/// Internal padding for the left detail box.
pub const DETAIL_PADDING: f32 = 20.0;

/// Border radius for both detail and list panels.
pub const PANEL_BORDER_RADIUS: f32 = 6.0;

// ── Shared Slider ─────────────────────────────────────────────────────────

/// Width of slider +/- buttons.
pub const SLIDER_BUTTON_SIZE: f32 = 30.0;

/// Border width for slider +/- buttons.
pub const SLIDER_BORDER_WIDTH: f32 = 2.0;

/// Width of the slider track.
pub const SLIDER_TRACK_WIDTH: f32 = 200.0;

/// Slider control background color (warm dark).
pub const SLIDER_BUTTON_BG: Color = Color::hsla(25.0, 0.12, 0.13, 1.0);

/// Slider control border color (purple).
pub const SLIDER_BUTTON_BORDER_COLOR: Color = Color::hsla(270.0, 0.40, 0.40, 1.0);

/// Font size for slider labels.
pub const SLIDER_LABEL_FONT_SIZE: f32 = 14.0;

/// Font size for slider +/- button text.
pub const SLIDER_BUTTON_FONT_SIZE: f32 = 13.0;

/// Spacing between slider control elements.
pub const SLIDER_GAP: f32 = 10.0;

/// Returns a color representing efficiency: red (0%) → green (99%) → gold (100%).
pub fn efficiency_color(eff: f32) -> Color {
    if eff >= 1.0 {
        Color::srgb(1.0, 0.85, 0.3)
    } else {
        let t = eff.clamp(0.0, 0.99);
        Color::srgb((1.0 - t) * 0.9, t * 0.85, 0.15)
    }
}
