use bevy::prelude::*;

use crate::ui::components::ButtonStyle;
use crate::ui::constants::{BUTTON_BG, BUTTON_BORDER, TEXT_BODY, TEXT_DISABLED, TEXT_PRIMARY};

// ---------------------------------------------------------------------------
// Arcane rune background
// ---------------------------------------------------------------------------
/// Overall opacity of the rune background shader layer (1.0 = fully opaque lines).
pub(crate) const RUNE_OPACITY: f32 = 1.0;
/// Font size for spell name text orbiting around the rune circles.
pub(crate) const RUNE_TEXT_SIZE: f32 = 30.0;
/// Alpha for orbiting spell name text.
pub(crate) const RUNE_TEXT_ALPHA: f32 = 0.12;
pub(crate) const TITLE_FONT_SIZE: f32 = 36.0;
pub(crate) const LEVEL_FONT_SIZE: f32 = 18.0;
pub(crate) const INSIGHT_FONT_SIZE: f32 = 22.0;

// ---------------------------------------------------------------------------
// Tab bar
// ---------------------------------------------------------------------------
pub(crate) use crate::ui::constants::{TAB_FONT_SIZE, TAB_HEIGHT, TAB_PADDING_H};

// ---------------------------------------------------------------------------
// Dual-panel layout
// ---------------------------------------------------------------------------
pub(crate) const LEFT_PANEL_PERCENT: f32 = 33.33;
pub(crate) const RIGHT_PANEL_PERCENT: f32 = 66.67;
pub(crate) const COLUMN_GAP: f32 = 12.0;
pub(crate) const SECTION_PADDING: f32 = 12.0;
pub(crate) const MARGIN_SMALL: f32 = 8.0;

// ---------------------------------------------------------------------------
// Endless tab
// ---------------------------------------------------------------------------
pub(crate) const STAT_SECTION_FONT_SIZE: f32 = 17.0;
pub(crate) const STAT_VALUE_FONT_SIZE: f32 = 14.0;

// ---------------------------------------------------------------------------
// Graph layout
// ---------------------------------------------------------------------------
pub(crate) const GRAPH_NODE_SIZE: f32 = 60.0;
pub(crate) const GRAPH_EDGE_THICKNESS: f32 = 4.0;
pub(crate) const GRAPH_FREE_NODE_SIZE: f32 = 40.0;
pub(crate) const GRAPH_EDGE_AVOIDANCE_MARGIN: f32 = 8.0;
pub(crate) const GRAPH_EDGE_CURVE_SEGMENTS: usize = 12;

// Graph zoom and animation
pub(crate) const GRAPH_ZOOM_MIN: f32 = 0.4;
pub(crate) const GRAPH_ZOOM_MAX: f32 = 2.0;
pub(crate) const GRAPH_ZOOM_SPEED: f32 = 0.1;
pub(crate) const GRAPH_ANIMATION_SPEED: f32 = 5.0;

// ---------------------------------------------------------------------------
// Detail panel
// ---------------------------------------------------------------------------
pub(crate) const DETAIL_PANEL_WIDTH: f32 = 340.0;
pub(crate) const DETAIL_PANEL_PADDING: f32 = 14.0;
pub(crate) const DETAIL_TITLE_FONT_SIZE: f32 = 20.0;
pub(crate) const DETAIL_TEXT_FONT_SIZE: f32 = 14.0;
pub(crate) const DETAIL_SMALL_FONT_SIZE: f32 = 12.0;

// Allocation slider (used in detail panel)
pub(crate) const SLIDER_TRACK_WIDTH: f32 = 260.0;
pub(crate) const SLIDER_TRACK_HEIGHT: f32 = 14.0;
pub(crate) const SLIDER_HANDLE_WIDTH: f32 = 7.0;
pub(crate) const SLIDER_HANDLE_HEIGHT: f32 = 24.0;

// ---------------------------------------------------------------------------
// Talent UI
// ---------------------------------------------------------------------------

// Talent progress bar
pub(crate) const TALENT_BAR_WIDTH: f32 = 12.0;

// Talent cards
pub(crate) const TALENT_CARD_WIDTH: f32 = 80.0;
pub(crate) const TALENT_CARD_HEIGHT: f32 = 50.0;
pub(crate) const TALENT_CARD_GAP: f32 = 8.0;
pub(crate) const TALENT_CARD_FONT: f32 = 9.0;
pub(crate) const TALENT_TIER_LABEL_FONT: f32 = 10.0;
pub(crate) const TALENT_DESC_FONT: f32 = 11.0;
pub(crate) const TALENT_DESC_FONT_SMALL: f32 = 9.0;
/// Character threshold above which talent/spell description text shrinks.
pub(crate) const DESC_SHRINK_THRESHOLD: usize = 80;

// Spell icon
pub(crate) const SPELL_ICON_SIZE: f32 = 34.0;

// ---------------------------------------------------------------------------
// Insight constellation
// ---------------------------------------------------------------------------
pub(crate) const INSIGHT_CONSTELLATION_OFFSET: Vec2 = Vec2::new(800.0, 0.0);

/// Scale the Study graph opens with — zoomed out enough to show both the
/// spell web and the insight constellation in one view.
pub(crate) const GRAPH_DEFAULT_SCALE: f32 = 0.55;
pub(crate) const INSIGHT_NODE_SIZE: f32 = 50.0;
pub(crate) const INSIGHT_ANCHOR_SIZE: f32 = 36.0;
pub(crate) const INSIGHT_NODE_RADIUS: f32 = 100.0;

// ---------------------------------------------------------------------------
// Buttons
// ---------------------------------------------------------------------------
pub(crate) const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 220.0,
    height: 55.0,
    border_width: 3.0,
    font_size: 16.0,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};

/// Greyed, unclickable variant of `BUTTON_STYLE` for a co-op start button that
/// is disabled until the guest readies ("Guest Not Ready"). Pair with a `()`
/// action so no system matches the click.
pub(crate) const GUEST_NOT_READY_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 220.0,
    height: 55.0,
    border_width: 3.0,
    font_size: 16.0,
    background: Color::hsla(0.0, 0.0, 0.18, 0.85),
    border: Color::hsla(0.0, 0.0, 0.30, 1.0),
    text_color: TEXT_DISABLED,
    text_shadow: false,
};

/// Spawns a co-op-gated mode-start button: the normal `(label, action, style)`
/// button, or a greyed unclickable "Guest Not Ready" button when a guest is
/// connected but not ready (`guest_pending == Some(false)`). Shared by the
/// Endless/Roguelite start buttons so the host can't start while the guest is
/// still readying.
pub(crate) fn spawn_coop_gated_button(
    parent: &mut ChildSpawnerCommands,
    guest_pending: Option<bool>,
    normal_label: &str,
    action: impl Bundle,
    normal_style: &ButtonStyle,
) {
    if guest_pending == Some(false) {
        crate::ui::systems::spawn_button(
            parent,
            "Guest Not Ready",
            (),
            &GUEST_NOT_READY_BUTTON_STYLE,
        );
    } else {
        crate::ui::systems::spawn_button(parent, normal_label, action, normal_style);
    }
}

pub(crate) const COMMIT_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 180.0,
    height: 50.0,
    border_width: 3.0,
    font_size: 16.0,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};

/// Small square +/- button used next to the study allocation slider to
/// let gamepad users adjust insight without dragging the handle.
pub(crate) const ALLOC_ADJUST_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 32.0,
    height: 32.0,
    border_width: 2.0,
    font_size: 18.0,
    background: Color::srgba(0.15, 0.15, 0.2, 0.85),
    border: Color::srgb(0.45, 0.45, 0.55),
    text_color: Color::srgb(0.95, 0.95, 0.95),
    text_shadow: false,
};

/// Insight allocation step size per +/- button press (spells and bonuses).
pub(crate) const ALLOC_ADJUST_STEP: u32 = 5;

pub(crate) use crate::ui::constants::BACK_BUTTON_STYLE;

/// Muted, smaller button for secondary actions like "Switch Wizard".
pub(crate) const SWITCH_WIZARD_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 200.0,
    height: 36.0,
    border_width: 1.0,
    font_size: 10.0,
    background: Color::hsla(25.0, 0.15, 0.07, 0.6),
    border: Color::hsla(270.0, 0.20, 0.25, 0.6),
    text_color: TEXT_BODY,
    text_shadow: true,
};

// ---------------------------------------------------------------------------
// Time Travel
// ---------------------------------------------------------------------------
pub(crate) const TIME_TRAVEL_LIST_MAX_HEIGHT: f32 = 200.0;
pub(crate) const TIME_TRAVEL_LEVEL_HEIGHT: f32 = 32.0;
pub(crate) const TIME_TRAVEL_LEVEL_FONT_SIZE: f32 = 14.0;

pub(crate) const START_TIME_TRAVEL_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 200.0,
    height: 40.0,
    border_width: 2.0,
    font_size: 14.0,
    background: Color::srgba(0.2, 0.1, 0.35, 0.75),
    border: Color::srgb(0.5, 0.3, 0.7),
    text_color: Color::srgb(0.9, 0.85, 1.0),
    text_shadow: true,
};

#[cfg(debug_assertions)]
pub(crate) const DEBUG_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 200.0,
    height: 50.0,
    border_width: 3.0,
    font_size: 14.0,
    background: Color::srgba(0.4, 0.1, 0.1, 0.75),
    border: Color::srgb(0.8, 0.2, 0.2),
    text_color: Color::srgb(1.0, 0.7, 0.7),
    text_shadow: true,
};
