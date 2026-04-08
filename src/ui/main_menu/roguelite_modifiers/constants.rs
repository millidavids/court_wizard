use bevy::prelude::*;

use crate::ui::components::ButtonStyle;
use crate::ui::constants::{
    BUTTON_BG, BUTTON_BG_SUBTLE, BUTTON_BORDER, BUTTON_BORDER_SUBTLE, TEXT_MUTED, TEXT_PRIMARY,
};

// ── General ─────────────────────────────────────────────────────────────────

/// Text color for modifier labels.
pub(super) const TEXT_COLOR: Color = TEXT_PRIMARY;

/// Subtitle / description color.
pub(super) const DESCRIPTION_COLOR: Color = TEXT_MUTED;

/// Spacing between sections.
pub(super) const MARGIN: f32 = 20.0;

/// Smaller spacing inside rows.
pub(super) const MARGIN_SMALL: f32 = 10.0;

// ── Left Panel (Run Summary) ────────────────────────────────────────────────

/// Title font size in the left panel.
pub(super) const SUMMARY_TITLE_FONT_SIZE: f32 = 16.0;

/// Font size for modifier entries in the summary.
pub(super) const SUMMARY_ITEM_FONT_SIZE: f32 = 11.0;

/// Color for modifier entries in the summary.
pub(super) const SUMMARY_ITEM_COLOR: Color = Color::hsla(270.0, 0.15, 0.70, 1.0);

/// Placeholder text when no modifiers are active.
pub(super) const SUMMARY_PLACEHOLDER_COLOR: Color = TEXT_MUTED;

/// Start Run button style.
pub(super) const START_RUN_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 260.0,
    height: 45.0,
    border_width: 2.0,
    font_size: 14.0,
    background: Color::hsla(270.0, 0.20, 0.18, 0.75),
    border: Color::hsla(270.0, 0.50, 0.40, 1.0),
    text_color: Color::hsla(270.0, 0.20, 0.85, 1.0),
    text_shadow: true,
};

/// Back button style.
pub(super) const BACK_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 260.0,
    height: 36.0,
    border_width: 1.0,
    font_size: 10.0,
    background: BUTTON_BG_SUBTLE,
    border: BUTTON_BORDER_SUBTLE,
    text_color: TEXT_MUTED,
    text_shadow: true,
};

// ── Right Panel (Configuration) ─────────────────────────────────────────────

/// Section header font size (e.g., "Toggle Modifiers").
pub(super) const SECTION_HEADER_FONT_SIZE: f32 = 13.0;

// ── Toggle Rows ─────────────────────────────────────────────────────────────

/// Background for a toggle row (locked).
pub(super) const TOGGLE_LOCKED_BG: Color = Color::hsla(0.0, 0.0, 0.08, 0.8);

/// Border for a toggle row (locked).
pub(super) const TOGGLE_LOCKED_BORDER: Color = Color::hsla(0.0, 0.0, 0.25, 0.6);

/// Background for an unlocked but disabled toggle row.
pub(super) const TOGGLE_OFF_BG: Color = Color::hsla(270.0, 0.10, 0.10, 0.6);

/// Border for an unlocked but disabled toggle row.
pub(super) const TOGGLE_OFF_BORDER: Color = Color::hsla(270.0, 0.10, 0.20, 0.5);

/// Background for an active (enabled) toggle row.
pub(super) const TOGGLE_ON_BG: Color = Color::hsla(270.0, 0.25, 0.16, 0.8);

/// Border for an active (enabled) toggle row.
pub(super) const TOGGLE_ON_BORDER: Color = Color::hsla(270.0, 0.40, 0.40, 0.8);

/// Font size for toggle names.
pub(super) const TOGGLE_NAME_FONT_SIZE: f32 = 12.0;

/// Font size for toggle description text.
pub(super) const TOGGLE_DESC_FONT_SIZE: f32 = 10.0;

/// Small button size (expand arrow, ON/OFF, unlock).
pub(super) const TOGGLE_SMALL_BUTTON_SIZE: f32 = 28.0;

/// Font size for small toggle buttons.
pub(super) const TOGGLE_SMALL_BUTTON_FONT_SIZE: f32 = 10.0;

// ── Confirmation Popup ──────────────────────────────────────────────────────

/// Popup overlay background.
pub(super) const POPUP_OVERLAY_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);

/// Popup box background.
pub(super) const POPUP_BOX_BG: Color = Color::hsla(20.0, 0.10, 0.10, 0.95);

/// Popup box border.
pub(super) const POPUP_BOX_BORDER: Color = Color::hsla(270.0, 0.55, 0.50, 1.0);

/// Popup text font size.
pub(super) const POPUP_FONT_SIZE: f32 = 14.0;

/// Confirm button style.
pub(super) const CONFIRM_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 120.0,
    height: 36.0,
    border_width: 2.0,
    font_size: 12.0,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};

/// Cancel button style.
pub(super) const CANCEL_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 120.0,
    height: 36.0,
    border_width: 1.0,
    font_size: 12.0,
    background: BUTTON_BG_SUBTLE,
    border: BUTTON_BORDER_SUBTLE,
    text_color: TEXT_MUTED,
    text_shadow: false,
};
