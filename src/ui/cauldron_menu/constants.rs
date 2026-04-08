use bevy::prelude::*;

use crate::ui::components::ButtonStyle;
use crate::ui::constants::{
    BUTTON_BG_SUBTLE, BUTTON_BORDER_SUBTLE, DETAIL_BG as GLOBAL_DETAIL_BG,
    DETAIL_BORDER as GLOBAL_DETAIL_BORDER, LIST_BG as GLOBAL_LIST_BG,
    LIST_BORDER as GLOBAL_LIST_BORDER, TEXT_BODY, TEXT_DISABLED, TEXT_MUTED, TEXT_PLACEHOLDER,
};

// ---------------------------------------------------------------------------
// Layout (matching spell book)
// ---------------------------------------------------------------------------

/// Padding around the entire layout.
pub const LAYOUT_PADDING: f32 = 30.0;

/// Gap between left panel and right list.
pub const COLUMN_GAP: f32 = 30.0;

/// Width of the left detail panel.
pub const LEFT_PANEL_WIDTH: f32 = 300.0;

/// Title font size.
pub const TITLE_FONT_SIZE: f32 = 20.0;

/// Title color — warm white.
pub const TITLE_COLOR: Color = Color::hsla(40.0, 0.10, 0.85, 1.0);

// ---------------------------------------------------------------------------
// Detail panel (left side)
// ---------------------------------------------------------------------------

/// Detail panel background color.
pub const DETAIL_BG: Color = GLOBAL_DETAIL_BG;

/// Detail panel border color — subtle gold.
pub const DETAIL_BORDER: Color = GLOBAL_DETAIL_BORDER;

/// Detail panel border width.
pub const DETAIL_BORDER_WIDTH: f32 = 1.0;

/// Detail panel border radius.
pub const DETAIL_BORDER_RADIUS: f32 = 6.0;

/// Detail panel internal padding.
pub const DETAIL_PADDING: f32 = 18.0;

/// Font size for section labels in the detail panel.
pub const DETAIL_LABEL_FONT_SIZE: f32 = 12.0;

/// Color for section labels — warm amber.
pub const DETAIL_LABEL_COLOR: Color = Color::hsla(270.0, 0.50, 0.60, 1.0);

/// Font size for effect preview text.
pub const EFFECT_PREVIEW_FONT_SIZE: f32 = 11.0;

/// Color for effect preview text.
pub const EFFECT_PREVIEW_COLOR: Color = Color::srgb(0.6, 0.8, 0.6);

/// Font size for brew info text (brew time, duration).
pub const BREW_INFO_FONT_SIZE: f32 = 11.0;

/// Color for brew info text.
pub const BREW_INFO_COLOR: Color = Color::srgb(0.7, 0.7, 0.5);

/// Color for ingredient count text when under limit.
pub const INGREDIENT_COUNT_COLOR: Color = Color::hsla(0.0, 0.0, 0.50, 1.0);

/// Color for ingredient count text when at limit.
pub const INGREDIENT_COUNT_FULL_COLOR: Color = Color::hsla(270.0, 0.50, 0.60, 1.0);

/// Brewing status font size.
pub const BREWING_STATUS_FONT_SIZE: f32 = 13.0;

/// Brewing status text color.
pub const BREWING_STATUS_COLOR: Color = Color::srgb(0.9, 0.7, 0.3);

/// Disabled/dimmed text color.
pub const DISABLED_TEXT_COLOR: Color = TEXT_DISABLED;

/// Placeholder text color for empty detail panel.
pub const PLACEHOLDER_TEXT_COLOR: Color = TEXT_PLACEHOLDER;

/// Font size for combo bonus text.
pub const COMBO_FONT_SIZE: f32 = 11.0;

/// Color for combo bonus text — gold.
pub const COMBO_COLOR: Color = Color::hsla(270.0, 0.50, 0.60, 1.0);

// ---------------------------------------------------------------------------
// Ingredient list (right side)
// ---------------------------------------------------------------------------

/// Background for the ingredient list area.
pub const LIST_BG: Color = GLOBAL_LIST_BG;

/// Border for the ingredient list area.
pub const LIST_BORDER: Color = GLOBAL_LIST_BORDER;

/// Border width for the list.
pub const LIST_BORDER_WIDTH: f32 = 1.0;

/// Border radius for the list.
pub const LIST_BORDER_RADIUS: f32 = 6.0;

/// Internal padding for the list.
pub const LIST_PADDING: f32 = 12.0;

/// Gap between items in each category column.
pub const LIST_ITEM_GAP: f32 = 10.0;

/// Gap between category columns.
pub const CATEGORY_GAP: f32 = 16.0;

/// Font size for category headers.
pub const CATEGORY_FONT_SIZE: f32 = 13.0;

/// Color for category header text.
pub const CATEGORY_COLOR: Color = TEXT_MUTED;

// ---------------------------------------------------------------------------
// Ingredient buttons
// ---------------------------------------------------------------------------

/// Ingredient button width.
pub const BUTTON_WIDTH: f32 = 160.0;

/// Ingredient button height.
pub const BUTTON_HEIGHT: f32 = 40.0;

/// Ingredient button border width.
pub const BUTTON_BORDER_WIDTH: f32 = 1.0;

/// Ingredient button font size.
pub const BUTTON_FONT_SIZE: f32 = 11.0;

/// Ingredient description font size.
pub const DESCRIPTION_FONT_SIZE: f32 = 9.0;

/// General text color.
pub const TEXT_COLOR: Color = TEXT_BODY;

/// Maximum number of ingredients allowed in a single brew.
pub const MAX_INGREDIENTS: usize = 3;

/// Button style for unselected ingredient toggle.
pub const INGREDIENT_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: BUTTON_WIDTH,
    height: BUTTON_HEIGHT,
    border_width: BUTTON_BORDER_WIDTH,
    font_size: BUTTON_FONT_SIZE,
    background: Color::hsla(270.0, 0.12, 0.11, 0.75),
    border: Color::hsla(270.0, 0.10, 0.20, 0.6),
    text_color: Color::hsla(0.0, 0.0, 0.85, 1.0),
    text_shadow: true,
};

/// Button style for selected ingredient toggle.
pub const INGREDIENT_SELECTED_STYLE: ButtonStyle = ButtonStyle {
    width: BUTTON_WIDTH,
    height: BUTTON_HEIGHT,
    border_width: BUTTON_BORDER_WIDTH,
    font_size: BUTTON_FONT_SIZE,
    background: Color::srgba(0.1, 0.25, 0.1, 0.75),
    border: Color::srgb(0.3, 0.7, 0.3),
    text_color: Color::srgb(0.7, 1.0, 0.7),
    text_shadow: true,
};

/// Button style for disabled (at limit) ingredient toggle.
pub const INGREDIENT_DISABLED_STYLE: ButtonStyle = ButtonStyle {
    width: BUTTON_WIDTH,
    height: BUTTON_HEIGHT,
    border_width: BUTTON_BORDER_WIDTH,
    font_size: BUTTON_FONT_SIZE,
    background: Color::srgba(0.06, 0.06, 0.06, 0.6),
    border: Color::srgba(0.15, 0.15, 0.15, 0.5),
    text_color: Color::srgb(0.55, 0.55, 0.55),
    text_shadow: true,
};

// ---------------------------------------------------------------------------
// Philosopher's Stone button
// ---------------------------------------------------------------------------

/// Button style for the Philosopher's Stone (unselected).
pub const STONE_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: BUTTON_WIDTH,
    height: BUTTON_HEIGHT,
    border_width: BUTTON_BORDER_WIDTH,
    font_size: BUTTON_FONT_SIZE,
    background: Color::srgba(0.18, 0.15, 0.05, 0.75),
    border: Color::srgb(0.55, 0.45, 0.12),
    text_color: Color::srgb(0.85, 0.75, 0.2),
    text_shadow: true,
};

/// Button style for the Philosopher's Stone (selected).
pub const STONE_SELECTED_STYLE: ButtonStyle = ButtonStyle {
    width: BUTTON_WIDTH,
    height: BUTTON_HEIGHT,
    border_width: BUTTON_BORDER_WIDTH,
    font_size: BUTTON_FONT_SIZE,
    background: Color::srgba(0.25, 0.20, 0.05, 0.75),
    border: Color::srgb(0.85, 0.75, 0.2),
    text_color: Color::srgb(1.0, 0.9, 0.3),
    text_shadow: true,
};

// ---------------------------------------------------------------------------
// Action buttons (Brew / Cancel / Close)
// ---------------------------------------------------------------------------

/// Button style for the brew button (enabled).
pub const BREW_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 160.0,
    height: 40.0,
    border_width: 1.0,
    font_size: 13.0,
    background: Color::hsla(270.0, 0.20, 0.18, 0.75),
    border: Color::hsla(270.0, 0.50, 0.40, 1.0),
    text_color: Color::hsla(270.0, 0.20, 0.85, 1.0),
    text_shadow: true,
};

/// Button style for the cancel brew button.
pub const CANCEL_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 160.0,
    height: 40.0,
    border_width: 1.0,
    font_size: 11.0,
    background: Color::srgba(0.25, 0.08, 0.08, 0.75),
    border: Color::srgb(0.5, 0.15, 0.15),
    text_color: Color::srgb(0.9, 0.4, 0.4),
    text_shadow: true,
};

/// Button style for the close button.
pub const CLOSE_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 120.0,
    height: 40.0,
    border_width: 1.0,
    font_size: 11.0,
    background: BUTTON_BG_SUBTLE,
    border: BUTTON_BORDER_SUBTLE,
    text_color: TEXT_MUTED,
    text_shadow: true,
};
