//! Global UI color palette and shared styling constants.
//!
//! Single source of truth for colors used across all UI modules.
//! Module-specific colors (element colors, team colors, bar fills, etc.)
//! remain in their respective module constants files.

use bevy::prelude::*;

// ── Text Hierarchy ──────────────────────────────────────────────────────────

/// Primary text color for titles, headings, and important text.
pub const TEXT_PRIMARY: Color = Color::hsla(0.0, 0.0, 0.92, 1.0);

/// Body text color for general content and descriptions.
pub const TEXT_BODY: Color = Color::hsla(0.0, 0.0, 0.85, 1.0);

/// Muted text color for labels, subtitles, and secondary information.
pub const TEXT_MUTED: Color = Color::hsla(0.0, 0.0, 0.65, 1.0);

/// Disabled text color for locked or unavailable items.
pub const TEXT_DISABLED: Color = Color::hsla(0.0, 0.0, 0.50, 1.0);

/// Placeholder text color for empty slots and invisible hints.
pub const TEXT_PLACEHOLDER: Color = Color::hsla(0.0, 0.0, 0.40, 1.0);

// ── Semantic Accents ────────────────────────────────────────────────────────

/// Arcane insight / information highlight color (light blue).
pub const INSIGHT_COLOR: Color = Color::srgb(0.6, 0.8, 1.0);

/// Gold accent for active/selected states and highlights.
pub const GOLD_ACCENT: Color = Color::hsla(40.0, 0.50, 0.45, 1.0);

/// Success state color (green).
pub const SUCCESS_COLOR: Color = Color::hsla(120.0, 0.6, 0.5, 1.0);

/// Error / danger state color (red).
pub const ERROR_COLOR: Color = Color::hsla(0.0, 0.6, 0.5, 1.0);

/// Warning / pending state color (gold).
pub const WARNING_COLOR: Color = Color::hsla(45.0, 0.6, 0.5, 1.0);

// ── Overlay / Container ─────────────────────────────────────────────────────

/// Transparent overlay behind content (the content box provides visual separation).
pub const OVERLAY_BG: Color = Color::NONE;

/// Translucent background for page content containers.
pub const CONTENT_BG: Color = Color::hsla(220.0, 0.08, 0.08, 0.55);

/// Subtle border for page content containers.
pub const CONTENT_BORDER: Color = Color::hsla(0.0, 0.0, 0.18, 0.4);

/// Background for inner scrollable areas within page containers.
pub const SCROLL_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.3);

/// Border for inner scrollable areas.
pub const SCROLL_BORDER: Color = Color::hsla(0.0, 0.0, 0.25, 0.5);

/// Shadow color for elevated UI panels.
pub const SHADOW_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.5);

/// Shadow color for inner scroll areas (slightly lighter).
pub const SCROLL_SHADOW_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.4);

/// Drop shadow color behind text.
pub const TEXT_SHADOW_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.5);

// ── Buttons (purple-tinted) ─────────────────────────────────────────────────

/// Standard button background color (semi-transparent dark purple).
pub const BUTTON_BG: Color = Color::hsla(270.0, 0.15, 0.12, 0.75);

/// Standard button border color (semi-transparent light purple).
pub const BUTTON_BORDER: Color = Color::hsla(270.0, 0.15, 0.25, 0.6);

/// Subtle/muted button background color.
pub const BUTTON_BG_SUBTLE: Color = Color::hsla(270.0, 0.10, 0.08, 0.6);

/// Subtle/muted button border color.
pub const BUTTON_BORDER_SUBTLE: Color = Color::hsla(270.0, 0.10, 0.18, 0.5);

// ── Detail Panels ───────────────────────────────────────────────────────────

/// Detail panel background (dark blue-tinted, semi-transparent).
pub const DETAIL_BG: Color = Color::hsla(220.0, 0.08, 0.10, 0.75);

/// Detail panel border (subtle gold).
pub const DETAIL_BORDER: Color = Color::hsla(40.0, 0.35, 0.30, 0.8);

/// List area background (darker blue-tinted, semi-transparent).
pub const LIST_BG: Color = Color::hsla(220.0, 0.08, 0.08, 0.75);

/// List area border (dark gray).
pub const LIST_BORDER: Color = Color::hsla(0.0, 0.0, 0.18, 0.8);

// ── Shared Slider ─────────────────────────────────────────────────────────

/// Width of slider +/- buttons.
pub const SLIDER_BUTTON_SIZE: f32 = 30.0;

/// Border width for slider +/- buttons.
pub const SLIDER_BORDER_WIDTH: f32 = 2.0;

/// Width of the slider track.
pub const SLIDER_TRACK_WIDTH: f32 = 200.0;

/// Slider control background color.
pub const SLIDER_BUTTON_BG: Color = Color::hsla(0.0, 0.0, 0.15, 1.0);

/// Slider control border color.
pub const SLIDER_BUTTON_BORDER_COLOR: Color = Color::hsla(0.0, 0.0, 0.4, 1.0);

/// Font size for slider labels.
pub const SLIDER_LABEL_FONT_SIZE: f32 = 14.0;

/// Font size for slider +/- button text.
pub const SLIDER_BUTTON_FONT_SIZE: f32 = 13.0;

/// Spacing between slider control elements.
pub const SLIDER_GAP: f32 = 10.0;
