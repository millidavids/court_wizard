use bevy::prelude::*;

use crate::ui::components::ButtonStyle;
use crate::ui::constants::{
    BUTTON_BG, BUTTON_BORDER, INSIGHT_COLOR as GLOBAL_INSIGHT, TEXT_BODY, TEXT_DISABLED,
    TEXT_PRIMARY,
};

// ---------------------------------------------------------------------------
// Shared layout
// ---------------------------------------------------------------------------
pub(super) const BACKGROUND_COLOR: Color = Color::srgb(0.08, 0.08, 0.1);

// ---------------------------------------------------------------------------
// Arcane rune background
// ---------------------------------------------------------------------------
/// Dark golden color for the arcane rune geometric pattern lines.
pub(super) const RUNE_COLOR: Color = Color::hsla(42.0, 0.5, 0.25, 1.0);
/// Overall opacity of the rune background shader layer (1.0 = fully opaque lines).
pub(super) const RUNE_OPACITY: f32 = 1.0;
/// Font size for spell name text orbiting around the rune circles.
pub(super) const RUNE_TEXT_SIZE: f32 = 30.0;
/// Alpha for orbiting spell name text.
pub(super) const RUNE_TEXT_ALPHA: f32 = 0.12;
pub(super) const TITLE_FONT_SIZE: f32 = 36.0;
pub(super) const LEVEL_FONT_SIZE: f32 = 18.0;
pub(super) const INSIGHT_FONT_SIZE: f32 = 22.0;

// ---------------------------------------------------------------------------
// Tab bar
// ---------------------------------------------------------------------------
pub(super) use crate::ui::constants::{
    ACTIVE_TAB_BG, ACTIVE_TAB_BORDER, INACTIVE_TAB_BG, TAB_BORDER, TAB_FONT_SIZE, TAB_HEIGHT,
    TAB_PADDING_H,
};
pub(super) const DISABLED_TAB_TEXT: Color = Color::hsla(0.0, 0.0, 0.45, 0.6);

// ---------------------------------------------------------------------------
// Dual-panel layout
// ---------------------------------------------------------------------------
pub(super) const LEFT_PANEL_PERCENT: f32 = 33.33;
pub(super) const RIGHT_PANEL_PERCENT: f32 = 66.67;
pub(super) const COLUMN_GAP: f32 = 12.0;
pub(super) const SECTION_PADDING: f32 = 12.0;
pub(super) const MARGIN_SMALL: f32 = 8.0;
pub(super) const SECTION_BG: Color = Color::hsla(20.0, 0.10, 0.08, 0.8);
pub(super) const DETAIL_BG: Color = Color::hsla(20.0, 0.08, 0.06, 0.9);
pub(super) const DETAIL_BORDER: Color = Color::hsla(42.0, 0.45, 0.30, 0.8);

// ---------------------------------------------------------------------------
// Endless tab
// ---------------------------------------------------------------------------
pub(super) const STAT_LABEL_COLOR: Color = TEXT_DISABLED;
pub(super) const STAT_SECTION_FONT_SIZE: f32 = 17.0;
pub(super) const STAT_SECTION_COLOR: Color = Color::hsla(0.0, 0.0, 0.95, 1.0);
pub(super) const STAT_VALUE_FONT_SIZE: f32 = 14.0;

// ---------------------------------------------------------------------------
// Shared colors
// ---------------------------------------------------------------------------
pub(super) const TITLE_COLOR: Color = TEXT_PRIMARY;
pub(super) const TEXT_COLOR: Color = TEXT_BODY;
pub(super) const INSIGHT_COLOR: Color = GLOBAL_INSIGHT;
pub(super) const LOCKED_TEXT_COLOR: Color = TEXT_DISABLED;
pub(super) const COMPLETED_COLOR: Color = Color::srgb(0.4, 0.9, 0.4);
pub(super) const AFFINITY_COLOR: Color = Color::srgb(1.0, 0.85, 0.3);
pub(super) const PENDING_COLOR: Color = Color::srgb(0.9, 0.7, 0.3);

// ---------------------------------------------------------------------------
// Element colors
// ---------------------------------------------------------------------------
pub(super) const FIRE_COLOR: Color = Color::srgb(1.0, 0.4, 0.2);
pub(super) const NATURE_COLOR: Color = Color::srgb(0.3, 0.85, 0.3);
pub(super) const ELECTRIC_COLOR: Color = Color::srgb(0.5, 0.7, 1.0);
pub(super) const NECROTIC_COLOR: Color = Color::srgb(0.7, 0.3, 0.8);
pub(super) const FORCE_COLOR: Color = Color::srgb(0.7, 0.7, 0.9);
pub(super) const FROST_COLOR: Color = Color::srgb(0.6, 0.85, 0.95);
pub(super) const POISON_COLOR: Color = crate::game::units::constants::POISON_EFFECT_COLOR;
pub(super) const POOP_COLOR: Color = crate::game::units::constants::EXCREMAGE_BROWN;

// ---------------------------------------------------------------------------
// Graph layout
// ---------------------------------------------------------------------------
pub(super) const GRAPH_NODE_SIZE: f32 = 60.0;
pub(super) const GRAPH_EDGE_THICKNESS: f32 = 4.0;
pub(super) const GRAPH_FREE_NODE_SIZE: f32 = 40.0;
pub(super) const GRAPH_EDGE_AVOIDANCE_MARGIN: f32 = 8.0;
pub(super) const GRAPH_EDGE_CURVE_SEGMENTS: usize = 12;

// Graph node colors
pub(super) const GRAPH_NODE_BG: Color = Color::srgb(0.12, 0.12, 0.15);
pub(super) const GRAPH_NODE_BORDER: Color = Color::srgb(0.3, 0.3, 0.35);
pub(super) const GRAPH_NODE_LOCKED_BG: Color = Color::srgb(0.08, 0.08, 0.08);
pub(super) const GRAPH_NODE_LOCKED_BORDER: Color = Color::srgb(0.2, 0.2, 0.2);
pub(super) const GRAPH_NODE_SELECTED_BORDER: Color = Color::srgb(0.9, 0.8, 0.2);
pub(super) const GRAPH_NODE_COMPLETED_BORDER: Color = Color::srgb(0.3, 0.7, 0.3);
pub(super) const GRAPH_NODE_FREE_BORDER: Color = Color::srgb(0.5, 0.5, 0.6);

// Graph edge colors
pub(super) const GRAPH_EDGE_COLOR: Color = Color::srgba(0.4, 0.4, 0.5, 0.7);
pub(super) const GRAPH_EDGE_LOCKED_COLOR: Color = Color::hsla(25.0, 0.10, 0.15, 0.4);

// Graph zoom and animation
pub(super) const GRAPH_ZOOM_MIN: f32 = 0.4;
pub(super) const GRAPH_ZOOM_MAX: f32 = 2.0;
pub(super) const GRAPH_ZOOM_SPEED: f32 = 0.1;
pub(super) const GRAPH_ANIMATION_SPEED: f32 = 5.0;

// Graph area
pub(super) const GRAPH_AREA_BG: Color = Color::srgba(0.04, 0.04, 0.06, 0.8);

// ---------------------------------------------------------------------------
// Detail panel
// ---------------------------------------------------------------------------
pub(super) const DETAIL_PANEL_WIDTH: f32 = 340.0;
pub(super) const DETAIL_PANEL_PADDING: f32 = 14.0;
pub(super) const DETAIL_TITLE_FONT_SIZE: f32 = 20.0;
pub(super) const DETAIL_TEXT_FONT_SIZE: f32 = 14.0;
pub(super) const DETAIL_SMALL_FONT_SIZE: f32 = 12.0;

// Progress fill color (used in node rings and unified slider)
pub(super) const PROGRESS_BAR_FILL: Color = Color::srgb(0.3, 0.6, 0.9);

// Allocation slider (used in detail panel)
pub(super) const SLIDER_TRACK_WIDTH: f32 = 260.0;
pub(super) const SLIDER_TRACK_HEIGHT: f32 = 14.0;
pub(super) const SLIDER_HANDLE_WIDTH: f32 = 7.0;
pub(super) const SLIDER_HANDLE_HEIGHT: f32 = 24.0;
pub(super) const SLIDER_TRACK_BG: Color = Color::srgb(0.2, 0.2, 0.2);
pub(super) const SLIDER_TRACK_BORDER: Color = Color::srgb(0.35, 0.35, 0.4);
pub(super) const SLIDER_FILL_COLOR: Color = Color::srgb(0.5, 0.7, 0.3);
pub(super) const SLIDER_HANDLE_COLOR: Color = Color::WHITE;

// ---------------------------------------------------------------------------
// Talent UI
// ---------------------------------------------------------------------------

// Talent progress bar
pub(super) const TALENT_BAR_WIDTH: f32 = 12.0;
pub(super) const TALENT_BAR_BG: Color = Color::srgb(0.12, 0.12, 0.12);
pub(super) const TALENT_BAR_FILL: Color = Color::srgb(0.6, 0.4, 0.9);

// Talent cards
pub(super) const TALENT_CARD_WIDTH: f32 = 80.0;
pub(super) const TALENT_CARD_HEIGHT: f32 = 50.0;
pub(super) const TALENT_CARD_GAP: f32 = 8.0;
pub(super) const TALENT_CARD_FONT: f32 = 9.0;
/// Purple accent for selected/active talents (matches insight constellation theme).
pub(super) const TALENT_ACTIVE_BG: Color = Color::srgb(0.18, 0.14, 0.25);
pub(super) const TALENT_ACTIVE_BORDER: Color = Color::srgb(0.6, 0.4, 0.9);
pub(super) const TALENT_UNLOCKED_BG: Color = Color::srgb(0.14, 0.14, 0.18);
pub(super) const TALENT_LOCKED_BG: Color = Color::srgb(0.07, 0.07, 0.07);
pub(super) const TALENT_LOCKED_BORDER: Color = Color::srgb(0.18, 0.18, 0.18);
pub(super) const TALENT_UNLOCKED_BORDER: Color = Color::srgb(0.3, 0.3, 0.35);
pub(super) const TALENT_TIER_LABEL_FONT: f32 = 10.0;
pub(super) const TALENT_DESC_FONT: f32 = 11.0;
pub(super) const TALENT_DESC_FONT_SMALL: f32 = 9.0;
/// Character threshold above which talent/spell description text shrinks.
pub(super) const DESC_SHRINK_THRESHOLD: usize = 80;

// Spell icon
pub(super) const SPELL_ICON_SIZE: f32 = 34.0;

// ---------------------------------------------------------------------------
// Insight constellation
// ---------------------------------------------------------------------------
pub(super) const INSIGHT_CONSTELLATION_OFFSET: Vec2 = Vec2::new(800.0, 0.0);

/// Scale the Study graph opens with — zoomed out enough to show both the
/// spell web and the insight constellation in one view.
pub(super) const GRAPH_DEFAULT_SCALE: f32 = 0.55;
pub(super) const INSIGHT_NODE_SIZE: f32 = 50.0;
pub(super) const INSIGHT_ANCHOR_SIZE: f32 = 36.0;
pub(super) const INSIGHT_NODE_RADIUS: f32 = 100.0;
pub(super) const INSIGHT_NODE_BG: Color = Color::srgb(0.1, 0.08, 0.15);
pub(super) const INSIGHT_NODE_BORDER: Color = Color::srgb(0.6, 0.4, 0.9);
pub(super) const INSIGHT_NODE_MAXED_BORDER: Color = Color::srgb(1.0, 0.85, 0.3);
pub(super) const INSIGHT_ANCHOR_BORDER: Color = Color::srgb(0.5, 0.3, 0.8);
pub(super) const INSIGHT_EDGE_COLOR: Color = Color::srgba(0.5, 0.3, 0.7, 0.6);
pub(super) const INSIGHT_PROGRESS_FILL: Color = Color::srgb(0.6, 0.4, 0.9);

// ---------------------------------------------------------------------------
// Buttons
// ---------------------------------------------------------------------------
pub(super) const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 220.0,
    height: 55.0,
    border_width: 3.0,
    font_size: 16.0,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};

pub(super) const COMMIT_BUTTON_STYLE: ButtonStyle = ButtonStyle {
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
pub(super) const ALLOC_ADJUST_BUTTON_STYLE: ButtonStyle = ButtonStyle {
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
pub(super) const ALLOC_ADJUST_STEP: u32 = 5;

pub(super) use crate::ui::constants::BACK_BUTTON_STYLE;

/// Muted, smaller button for secondary actions like "Switch Wizard".
pub(super) const SWITCH_WIZARD_BUTTON_STYLE: ButtonStyle = ButtonStyle {
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
pub(super) const TIME_TRAVEL_SECTION_BG: Color = Color::srgb(0.06, 0.04, 0.1);
pub(super) const TIME_TRAVEL_SECTION_BORDER: Color = Color::srgb(0.3, 0.2, 0.5);
pub(super) const TIME_TRAVEL_LIST_MAX_HEIGHT: f32 = 200.0;
pub(super) const TIME_TRAVEL_LEVEL_HEIGHT: f32 = 32.0;
pub(super) const TIME_TRAVEL_BOSS_COLOR: Color = Color::srgb(1.0, 0.7, 0.2);
pub(super) const TIME_TRAVEL_HOVER_BG: Color = Color::srgba(1.0, 0.95, 0.7, 0.1);
pub(super) const TIME_TRAVEL_SELECTED_BG: Color = Color::srgba(1.0, 0.85, 0.3, 0.15);
pub(super) const TIME_TRAVEL_SELECTED_TEXT: Color = Color::srgb(1.0, 0.85, 0.3);
pub(super) const TIME_TRAVEL_LEVEL_FONT_SIZE: f32 = 14.0;

pub(super) const START_TIME_TRAVEL_BUTTON_STYLE: ButtonStyle = ButtonStyle {
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
pub(super) const DEBUG_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 200.0,
    height: 50.0,
    border_width: 3.0,
    font_size: 14.0,
    background: Color::srgba(0.4, 0.1, 0.1, 0.75),
    border: Color::srgb(0.8, 0.2, 0.2),
    text_color: Color::srgb(1.0, 0.7, 0.7),
    text_shadow: true,
};
