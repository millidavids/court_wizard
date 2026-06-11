use bevy::prelude::*;

use crate::ui::components::ButtonStyle;
use crate::ui::constants::{TEXT_MUTED, TEXT_PRIMARY};

use super::super::constants::SWITCH_WIZARD_BUTTON_STYLE;

/// Maximum seed value (10 digits, fits in the text box).
pub(crate) const MAX_SEED: u64 = 10_000_000_000;
/// Maximum number of characters in the seed input field.
pub(crate) const MAX_SEED_CHARS: usize = 10;

/// Text color for modifier labels.
pub(crate) const LABEL_COLOR: Color = TEXT_PRIMARY;

/// Subtitle / description color.
pub(crate) const DESCRIPTION_COLOR: Color = TEXT_MUTED;

/// Spacing between sections.
pub(crate) const SECTION_MARGIN: f32 = 20.0;

/// Smaller spacing inside rows.
pub(crate) const ROW_GAP: f32 = 10.0;

/// Section header font size.
pub(crate) const SECTION_HEADER_FONT_SIZE: f32 = 13.0;

/// Summary title font size in the left panel.
pub(crate) const SUMMARY_TITLE_FONT_SIZE: f32 = 16.0;

/// Font size for modifier entries in the summary.
pub(crate) const SUMMARY_ITEM_FONT_SIZE: f32 = 11.0;

/// Color for modifier entries in the summary.
pub(crate) const SUMMARY_ITEM_COLOR: Color = Color::hsla(270.0, 0.15, 0.70, 1.0);

/// Placeholder text color when no modifiers are active.
pub(crate) const SUMMARY_PLACEHOLDER_COLOR: Color = TEXT_MUTED;

// Toggle row colors
pub(crate) const TOGGLE_LOCKED_BG: Color = Color::hsla(0.0, 0.0, 0.08, 0.8);
pub(crate) const TOGGLE_LOCKED_BORDER: Color = Color::hsla(0.0, 0.0, 0.25, 0.6);
pub(crate) const TOGGLE_OFF_BG: Color = Color::hsla(270.0, 0.10, 0.10, 0.6);
pub(crate) const TOGGLE_OFF_BORDER: Color = Color::hsla(270.0, 0.10, 0.20, 0.5);
pub(crate) const TOGGLE_ON_BG: Color = Color::hsla(270.0, 0.25, 0.16, 0.8);
pub(crate) const TOGGLE_ON_BORDER: Color = Color::hsla(270.0, 0.40, 0.40, 0.8);
pub(crate) const TOGGLE_NAME_FONT_SIZE: f32 = 12.0;
pub(crate) const TOGGLE_DESC_FONT_SIZE: f32 = 10.0;
pub(crate) const TOGGLE_SMALL_BUTTON_FONT_SIZE: f32 = 10.0;

// Confirmation popup
pub(crate) const POPUP_OVERLAY_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);
pub(crate) const POPUP_BOX_BG: Color = Color::hsla(20.0, 0.10, 0.10, 0.95);
pub(crate) const POPUP_BOX_BORDER: Color = Color::hsla(270.0, 0.55, 0.50, 1.0);
pub(crate) const POPUP_FONT_SIZE: f32 = 14.0;

/// Start Run button style.
pub(crate) const START_RUN_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 260.0,
    height: 45.0,
    border_width: 2.0,
    font_size: 14.0,
    background: Color::hsla(270.0, 0.20, 0.18, 0.75),
    border: Color::hsla(270.0, 0.50, 0.40, 1.0),
    text_color: Color::hsla(270.0, 0.20, 0.85, 1.0),
    text_shadow: true,
};

/// Change Wizard Type button style — reuse shared muted style.
pub(crate) const CHANGE_WIZARD_BUTTON_STYLE: ButtonStyle = SWITCH_WIZARD_BUTTON_STYLE;

/// Continue Run button style.
pub(crate) const CONTINUE_RUN_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 260.0,
    height: 45.0,
    border_width: 2.0,
    font_size: 14.0,
    background: Color::hsla(120.0, 0.20, 0.18, 0.75),
    border: Color::hsla(120.0, 0.50, 0.40, 1.0),
    text_color: Color::hsla(120.0, 0.20, 0.85, 1.0),
    text_shadow: true,
};

/// End Run button style.
pub(crate) const END_RUN_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 260.0,
    height: 36.0,
    border_width: 1.0,
    font_size: 10.0,
    background: Color::hsla(0.0, 0.20, 0.14, 0.6),
    border: Color::hsla(0.0, 0.40, 0.35, 0.6),
    text_color: Color::hsla(0.0, 0.15, 0.70, 1.0),
    text_shadow: true,
};

/// Confirm button style for popups.
pub(crate) const CONFIRM_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 120.0,
    height: 36.0,
    border_width: 2.0,
    font_size: 12.0,
    background: Color::hsla(25.0, 0.18, 0.14, 0.85),
    border: Color::hsla(25.0, 0.30, 0.30, 1.0),
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};

/// Cancel button style for popups.
pub(crate) const CANCEL_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 120.0,
    height: 36.0,
    border_width: 1.0,
    font_size: 12.0,
    background: Color::hsla(25.0, 0.15, 0.07, 0.6),
    border: Color::hsla(270.0, 0.20, 0.25, 0.6),
    text_color: TEXT_MUTED,
    text_shadow: false,
};

/// Active run stats font sizes.
pub(crate) const RUN_STATS_LABEL_FONT: f32 = 12.0;
pub(crate) const RUN_STATS_VALUE_FONT: f32 = 11.0;
pub(crate) const RUN_STATS_VALUE_COLOR: Color = Color::hsla(270.0, 0.15, 0.70, 1.0);
