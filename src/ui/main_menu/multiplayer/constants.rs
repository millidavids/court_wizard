//! Multiplayer screen styling constants.

use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

/// Background color for multiplayer screen buttons.
pub(super) const BUTTON_BACKGROUND: Color = Color::hsla(0.0, 0.0, 0.15, 1.0);

/// Border color for multiplayer screen buttons.
pub(super) const BUTTON_BORDER: Color = Color::hsla(0.0, 0.0, 0.3, 1.0);

/// Width for multiplayer screen buttons in pixels.
pub(super) const BUTTON_WIDTH: f32 = 250.0;

/// Height for multiplayer screen buttons in pixels.
pub(super) const BUTTON_HEIGHT: f32 = 65.0;

/// Border width for multiplayer screen buttons in pixels.
pub(super) const BUTTON_BORDER_WIDTH: f32 = 3.0;

/// Font size for multiplayer screen button text.
pub(super) const BUTTON_FONT_SIZE: f32 = 20.0;

/// Font size for multiplayer screen title text.
pub(super) const TITLE_FONT_SIZE: f32 = 48.0;

/// Font size for status text.
pub(super) const STATUS_FONT_SIZE: f32 = 18.0;

/// Font size for the code display text.
pub(super) const CODE_FONT_SIZE: f32 = 12.0;

/// Text color for multiplayer screen UI elements.
pub(super) const TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.9, 1.0);

/// Accent color for success/connected status.
pub(super) const SUCCESS_COLOR: Color = Color::hsla(120.0, 0.6, 0.5, 1.0);

/// Accent color for error/failed status.
pub(super) const ERROR_COLOR: Color = Color::hsla(0.0, 0.6, 0.5, 1.0);

/// Accent color for waiting/connecting status.
pub(super) const WAITING_COLOR: Color = Color::hsla(45.0, 0.6, 0.5, 1.0);

/// Margin between multiplayer screen UI elements in pixels.
pub(super) const MARGIN: f32 = 20.0;

/// Button style configuration for the multiplayer screen.
pub(super) const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: BUTTON_WIDTH,
    height: BUTTON_HEIGHT,
    border_width: BUTTON_BORDER_WIDTH,
    font_size: BUTTON_FONT_SIZE,
    background: BUTTON_BACKGROUND,
    border: BUTTON_BORDER,
    text_color: TEXT_COLOR,
};

// ===== Wizard Select Phase Styling =====

/// Background color for wizard cards.
pub(super) const CARD_BG: Color = Color::hsla(0.0, 0.0, 0.12, 1.0);

/// Border color for selected wizard card.
pub(super) const CARD_SELECTED_BORDER: Color = Color::hsla(45.0, 0.8, 0.5, 1.0);

/// Border color for unselected wizard card.
pub(super) const CARD_BORDER: Color = Color::hsla(0.0, 0.0, 0.3, 1.0);

/// Background color for locked wizard cards.
pub(super) const LOCKED_CARD_BG: Color = Color::hsla(0.0, 0.0, 0.08, 1.0);

/// Text color for locked content.
pub(super) const LOCKED_TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.4, 1.0);

/// Font size for wizard card names.
pub(super) const CARD_NAME_FONT_SIZE: f32 = 16.0;

/// Font size for wizard card descriptions.
pub(super) const CARD_DESC_FONT_SIZE: f32 = 12.0;

/// Width and height for wizard cards.
pub(super) const CARD_SIZE: f32 = 120.0;

/// Font size for spell slot text.
pub(super) const SPELL_FONT_SIZE: f32 = 11.0;

/// Background for active spell slots.
pub(super) const SPELL_SLOT_BG: Color = Color::hsla(0.0, 0.0, 0.18, 1.0);

/// Border for active spell slots.
pub(super) const SPELL_SLOT_BORDER: Color = Color::hsla(220.0, 0.5, 0.5, 1.0);

/// Background for empty spell slots.
pub(super) const SPELL_SLOT_EMPTY_BG: Color = Color::hsla(0.0, 0.0, 0.1, 1.0);

/// Ready button style.
pub(super) const READY_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 200.0,
    height: 50.0,
    border_width: BUTTON_BORDER_WIDTH,
    font_size: BUTTON_FONT_SIZE,
    background: Color::hsla(120.0, 0.4, 0.2, 1.0),
    border: Color::hsla(120.0, 0.6, 0.4, 1.0),
    text_color: TEXT_COLOR,
};
