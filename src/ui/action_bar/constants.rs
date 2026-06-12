use bevy::prelude::*;

use crate::ui::components::ButtonStyle;
use crate::ui::constants::{BUTTON_BG, BUTTON_BORDER};

/// Width of each action bar slot button.
pub(super) const SLOT_WIDTH: f32 = 50.0;

/// Height of each action bar slot button.
pub(super) const SLOT_HEIGHT: f32 = 50.0;

/// Gap between action bar slots.
pub(super) const SLOT_GAP: f32 = 4.0;

/// Font size for spell name text in action bar slots.
/// Spell names are no longer rendered in slots (icons are the identity);
/// this value is kept for the radial animation scale calculation.
pub(super) const SPELL_NAME_FONT_SIZE: f32 = 7.0;

/// Font size for hotkey indicator text.
pub(super) const HOTKEY_FONT_SIZE: f32 = 8.0;

/// Button style for action bar slots.
pub(super) const SLOT_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: SLOT_WIDTH,
    height: SLOT_HEIGHT,
    border_width: 2.0,
    font_size: SPELL_NAME_FONT_SIZE,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: Color::WHITE,
    text_shadow: true,
};

/// Lighter slot background used for the Warglock action bar so the dark
/// gun icons stay legible against it.
pub(super) const WARGLOCK_SLOT_BACKGROUND: Color = Color::srgba(0.55, 0.50, 0.48, 0.85);

/// Bottom margin for the action bar from screen edge.
pub(super) const ACTION_BAR_BOTTOM_MARGIN: f32 = 15.0;

/// Left margin for the action bar from screen edge.
pub(super) const ACTION_BAR_LEFT_MARGIN: f32 = 15.0;

/// Size of spell icon images in action bar slots.
pub(super) const SPELL_ICON_SIZE: f32 = 28.0;

// ===== Radial layout (active when a gamepad is the input device) =====

/// Horizontal center (from screen left) of the radial ring. Positioned to
/// the right of the wizard sprite so the slots don't overlap the hero.
pub(super) const RADIAL_CENTER_LEFT: f32 = 300.0;
/// Vertical center (from screen bottom) of the radial ring.
pub(super) const RADIAL_CENTER_BOTTOM: f32 = 70.0;
/// Distance from ring center to each slot's center.
pub(super) const RADIAL_RING_RADIUS: f32 = 60.0;
/// Scale applied to each slot button when fully in radial mode (1.0 = same
/// as linear). Shrunk so the ring reads as a compact secondary HUD rather
/// than five full-size buttons.
pub(super) const RADIAL_SLOT_SCALE: f32 = 0.65;
/// Seconds for a full linear ↔ radial transition.
pub(super) const RADIAL_TRANSITION_SECS: f32 = 0.35;
/// Duration of the commit flash.
pub(super) const RADIAL_COMMIT_FLASH_SECS: f32 = 0.18;
/// Border color shown on the radial slot the right stick is currently pointing at.
pub(super) const RADIAL_HOVER_COLOR: Color = Color::srgba(1.0, 0.95, 0.4, 1.0);
/// Border color for the gunslinger's currently selected gun slot.
pub(super) const GUNSLINGER_SELECTED_COLOR: Color = Color::srgb(1.0, 0.8, 0.2);
