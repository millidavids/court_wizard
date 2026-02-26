//! Spell book UI styling constants.

use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

/// Semi-transparent background for the full-screen overlay.
pub const BACKGROUND_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.85);

/// Padding around the entire spell book layout.
pub const LAYOUT_PADDING: f32 = 30.0;

/// Gap between left panel and right list.
pub const COLUMN_GAP: f32 = 30.0;

/// Width of the left detail panel.
pub const LEFT_PANEL_WIDTH: f32 = 300.0;

// ---------------------------------------------------------------------------
// Detail panel (left side)
// ---------------------------------------------------------------------------

/// Detail panel background color.
pub const DETAIL_BG: Color = Color::hsla(220.0, 0.08, 0.10, 1.0);

/// Detail panel border color — subtle gold.
pub const DETAIL_BORDER: Color = Color::hsla(40.0, 0.35, 0.30, 1.0);

/// Detail panel border width.
pub const DETAIL_BORDER_WIDTH: f32 = 1.0;

/// Detail panel border radius.
pub const DETAIL_BORDER_RADIUS: f32 = 6.0;

/// Detail panel internal padding.
pub const DETAIL_PADDING: f32 = 20.0;

/// Font size for spell name in the detail panel.
pub const DETAIL_NAME_FONT_SIZE: f32 = 14.0;

/// Color for spell name text — warm white.
pub const DETAIL_NAME_COLOR: Color = Color::hsla(40.0, 0.10, 0.85, 1.0);

/// Font size for damage type label.
pub const DETAIL_TYPE_FONT_SIZE: f32 = 7.0;

/// Color for damage type label — warm amber.
pub const DETAIL_TYPE_COLOR: Color = Color::hsla(40.0, 0.55, 0.55, 1.0);

/// Font size for description text.
pub const DETAIL_DESC_FONT_SIZE: f32 = 7.0;

/// Color for description text.
pub const DETAIL_DESC_COLOR: Color = Color::hsla(0.0, 0.0, 0.65, 1.0);

/// Font size for instructions text.
pub const DETAIL_INSTRUCTIONS_FONT_SIZE: f32 = 7.0;

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
pub const HOTKEY_INACTIVE_BG: Color = Color::hsla(220.0, 0.08, 0.12, 1.0);

/// Border for inactive hotkey box.
pub const HOTKEY_INACTIVE_BORDER: Color = Color::hsla(0.0, 0.0, 0.25, 1.0);

/// Text color for inactive hotkey.
pub const HOTKEY_INACTIVE_TEXT: Color = Color::hsla(0.0, 0.0, 0.40, 1.0);

/// Background for active hotkey box (this spell is assigned to this slot).
pub const HOTKEY_ACTIVE_BG: Color = Color::hsla(40.0, 0.20, 0.18, 1.0);

/// Border for active hotkey box.
pub const HOTKEY_ACTIVE_BORDER: Color = Color::hsla(40.0, 0.50, 0.45, 1.0);

/// Text color for active hotkey.
pub const HOTKEY_ACTIVE_TEXT: Color = Color::hsla(40.0, 0.20, 0.85, 1.0);

// ---------------------------------------------------------------------------
// Spell list (right side)
// ---------------------------------------------------------------------------

/// Background for the spell list scroll area.
pub const LIST_BG: Color = Color::hsla(220.0, 0.08, 0.08, 1.0);

/// Border for the spell list scroll area.
pub const LIST_BORDER: Color = Color::hsla(0.0, 0.0, 0.18, 1.0);

/// Border width for the spell list.
pub const LIST_BORDER_WIDTH: f32 = 1.0;

/// Border radius for the spell list.
pub const LIST_BORDER_RADIUS: f32 = 6.0;

/// Internal padding for the spell list.
pub const LIST_PADDING: f32 = 12.0;

/// Gap between items in the spell list.
pub const LIST_ITEM_GAP: f32 = 4.0;

/// Gap between category sections.
pub const CATEGORY_GAP: f32 = 16.0;

/// Font size for category headers.
pub const CATEGORY_FONT_SIZE: f32 = 7.0;

/// Color for category header text.
pub const CATEGORY_COLOR: Color = Color::hsla(0.0, 0.0, 0.40, 1.0);

/// Width of spell buttons in the list.
pub const SPELL_BUTTON_WIDTH: f32 = 220.0;

/// Height of spell buttons in the list.
pub const SPELL_BUTTON_HEIGHT: f32 = 40.0;

/// Background for spell buttons.
pub const SPELL_BUTTON_BG: Color = Color::hsla(220.0, 0.08, 0.11, 1.0);

/// Border for spell buttons.
pub const SPELL_BUTTON_BORDER: Color = Color::hsla(0.0, 0.0, 0.20, 1.0);

/// Border for the currently selected spell button — gold accent.
pub const SPELL_BUTTON_SELECTED_BORDER: Color = Color::hsla(40.0, 0.50, 0.45, 1.0);

/// Text color for spell buttons.
pub const SPELL_BUTTON_TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.85, 1.0);

/// Font size for spell button text.
pub const SPELL_BUTTON_FONT_SIZE: f32 = 8.0;

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
pub const LABEL_COLOR: Color = Color::hsla(0.0, 0.0, 0.40, 1.0);

// ---------------------------------------------------------------------------
// Buttons (Select / Close)
// ---------------------------------------------------------------------------

/// Button style for the "Select" button.
pub const SELECT_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 130.0,
    height: 36.0,
    border_width: 1.0,
    font_size: 8.0,
    background: Color::hsla(40.0, 0.20, 0.18, 1.0),
    border: Color::hsla(40.0, 0.40, 0.35, 1.0),
    text_color: Color::hsla(40.0, 0.20, 0.85, 1.0),
};

/// Button style for the "Close" button.
pub const CLOSE_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 100.0,
    height: 36.0,
    border_width: 1.0,
    font_size: 7.0,
    background: Color::hsla(0.0, 0.0, 0.10, 1.0),
    border: Color::hsla(0.0, 0.0, 0.22, 1.0),
    text_color: Color::hsla(0.0, 0.0, 0.50, 1.0),
};
