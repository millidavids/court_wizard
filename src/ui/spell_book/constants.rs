//! Spell book UI styling constants.

use bevy::prelude::*;

use crate::ui::components::ButtonStyle;
use crate::ui::constants::{BUTTON_BG_SUBTLE, BUTTON_BORDER_SUBTLE, GOLD_ACCENT, TEXT_BODY, TEXT_MUTED};

// Re-export shared layout constants used in systems.rs
pub use crate::ui::constants::{DETAIL_PADDING, LEFT_PANEL_WIDTH};

/// Font size for spell name in the detail panel.
pub const DETAIL_NAME_FONT_SIZE: f32 = 14.0;

/// Color for spell name text — warm white.
pub const DETAIL_NAME_COLOR: Color = Color::hsla(40.0, 0.10, 0.85, 1.0);

/// Font size for damage type label.
pub const DETAIL_TYPE_FONT_SIZE: f32 = 7.0;

/// Color for damage type label — purple accent.
pub const DETAIL_TYPE_COLOR: Color = Color::hsla(270.0, 0.50, 0.60, 1.0);

/// Font size for description text.
pub const DETAIL_DESC_FONT_SIZE: f32 = 11.0;

/// Color for description text.
pub const DETAIL_DESC_COLOR: Color = TEXT_MUTED;

/// Font size for instructions text.
pub const DETAIL_INSTRUCTIONS_FONT_SIZE: f32 = 11.0;

/// Color for instructions text — slightly warm.
pub const DETAIL_INSTRUCTIONS_COLOR: Color = Color::hsla(40.0, 0.15, 0.55, 1.0);

// ---------------------------------------------------------------------------
// Hotkey boxes
// ---------------------------------------------------------------------------

/// Size of each hotkey slot box.
pub const HOTKEY_BOX_SIZE: f32 = 36.0;

/// Gap between hotkey boxes.
pub const HOTKEY_BOX_GAP: f32 = 6.0;

/// Font size for the hotkey number label.
pub const HOTKEY_FONT_SIZE: f32 = 8.0;

/// Background for inactive hotkey box.
pub const HOTKEY_INACTIVE_BG: Color = Color::hsla(20.0, 0.10, 0.10, 0.75);

/// Border for inactive hotkey box.
pub const HOTKEY_INACTIVE_BORDER: Color = Color::hsla(0.0, 0.0, 0.25, 0.6);

/// Text color for inactive hotkey.
pub const HOTKEY_INACTIVE_TEXT: Color = TEXT_MUTED;

/// Background for active hotkey box (this spell is assigned to this slot).
pub const HOTKEY_ACTIVE_BG: Color = Color::hsla(270.0, 0.20, 0.18, 1.0);

/// Border for active hotkey box.
pub const HOTKEY_ACTIVE_BORDER: Color = Color::hsla(270.0, 0.50, 0.45, 1.0);

/// Text color for active hotkey.
pub const HOTKEY_ACTIVE_TEXT: Color = Color::hsla(270.0, 0.20, 0.85, 1.0);

// ---------------------------------------------------------------------------
// Spell list (right side)
// ---------------------------------------------------------------------------

/// Gap between items in the spell list.
pub const LIST_ITEM_GAP: f32 = 16.0;

/// Font size for category headers.
pub const CATEGORY_FONT_SIZE: f32 = 9.0;

/// Height of spell buttons in the list.
pub const SPELL_BUTTON_HEIGHT: f32 = 40.0;

/// Background for spell buttons (warm dark brown).
pub const SPELL_BUTTON_BG: Color = Color::hsla(20.0, 0.12, 0.09, 0.75);

/// Border for spell buttons (warm bronze).
pub const SPELL_BUTTON_BORDER: Color = Color::hsla(270.0, 0.20, 0.22, 0.6);

/// Border for the currently selected spell button — gold accent.
pub const SPELL_BUTTON_SELECTED_BORDER: Color = GOLD_ACCENT;

/// Text color for spell buttons.
pub const SPELL_BUTTON_TEXT_COLOR: Color = TEXT_BODY;

/// Font size for spell button text.
pub const SPELL_BUTTON_FONT_SIZE: f32 = 10.0;

/// Border width for spell buttons.
pub const SPELL_BUTTON_BORDER_WIDTH: f32 = 1.0;

/// Size of spell icon images in spell book buttons.
pub const SPELL_ICON_SIZE: f32 = 24.0;

// ---------------------------------------------------------------------------
// Label
// ---------------------------------------------------------------------------

/// Font size for section labels ("Assign Hotkey").
pub const LABEL_FONT_SIZE: f32 = 7.0;

/// Color for section labels.
pub const LABEL_COLOR: Color = TEXT_MUTED;

// ---------------------------------------------------------------------------
// Buttons (Select / Close)
// ---------------------------------------------------------------------------

/// Button style for the "Select" button.
pub const SELECT_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 130.0,
    height: 36.0,
    border_width: 1.0,
    font_size: 8.0,
    background: Color::hsla(270.0, 0.20, 0.18, 0.75),
    border: Color::hsla(270.0, 0.50, 0.40, 1.0),
    text_color: Color::hsla(270.0, 0.20, 0.85, 1.0),
    text_shadow: true,
};

/// Button style for the "Close" button.
pub const CLOSE_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 100.0,
    height: 36.0,
    border_width: 1.0,
    font_size: 7.0,
    background: BUTTON_BG_SUBTLE,
    border: BUTTON_BORDER_SUBTLE,
    text_color: TEXT_MUTED,
    text_shadow: true,
};
