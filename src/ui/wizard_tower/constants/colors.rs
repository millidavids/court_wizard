use bevy::prelude::*;

use crate::ui::constants::{TEXT_BODY, TEXT_DISABLED, TEXT_PRIMARY};

// ---------------------------------------------------------------------------
// Shared layout
// ---------------------------------------------------------------------------
pub(crate) const BACKGROUND_COLOR: Color = Color::srgb(0.08, 0.08, 0.1);

// ---------------------------------------------------------------------------
// Arcane rune background
// ---------------------------------------------------------------------------
/// Dark golden color for the arcane rune geometric pattern lines.
pub(crate) const RUNE_COLOR: Color = Color::hsla(42.0, 0.5, 0.25, 1.0);

// ---------------------------------------------------------------------------
// Tab bar
// ---------------------------------------------------------------------------
pub(crate) use crate::ui::constants::{
    ACTIVE_TAB_BG, ACTIVE_TAB_BORDER, INACTIVE_TAB_BG, TAB_BORDER,
};
pub(crate) const DISABLED_TAB_TEXT: Color = Color::hsla(0.0, 0.0, 0.45, 0.6);
/// Greyed background + border for a disabled tab (guest-locked Endless/Roguelite,
/// or VS before a connection), so it reads as unavailable, not just dim text.
pub(crate) const DISABLED_TAB_BG: Color = Color::hsla(0.0, 0.0, 0.12, 0.6);
pub(crate) const DISABLED_TAB_BORDER: Color = Color::hsla(0.0, 0.0, 0.28, 0.7);

// ---------------------------------------------------------------------------
// Dual-panel layout
// ---------------------------------------------------------------------------
pub(crate) const SECTION_BG: Color = Color::hsla(20.0, 0.10, 0.08, 0.8);
pub(crate) const DETAIL_BG: Color = Color::hsla(20.0, 0.08, 0.06, 0.9);
pub(crate) const DETAIL_BORDER: Color = Color::hsla(42.0, 0.45, 0.30, 0.8);

// ---------------------------------------------------------------------------
// Endless tab
// ---------------------------------------------------------------------------
pub(crate) const STAT_LABEL_COLOR: Color = TEXT_DISABLED;
pub(crate) const STAT_SECTION_COLOR: Color = Color::hsla(0.0, 0.0, 0.95, 1.0);

// ---------------------------------------------------------------------------
// Shared colors
// ---------------------------------------------------------------------------
pub(crate) const TITLE_COLOR: Color = TEXT_PRIMARY;
pub(crate) const TEXT_COLOR: Color = TEXT_BODY;
pub(crate) const INSIGHT_COLOR: Color = crate::ui::constants::INSIGHT_COLOR;
pub(crate) const LOCKED_TEXT_COLOR: Color = TEXT_DISABLED;
pub(crate) const COMPLETED_COLOR: Color = Color::srgb(0.4, 0.9, 0.4);
pub(crate) const AFFINITY_COLOR: Color = Color::srgb(1.0, 0.85, 0.3);
pub(crate) const PENDING_COLOR: Color = Color::srgb(0.9, 0.7, 0.3);

// Element colors live on `DamageType::element_color()` (src/game/units/damage.rs)
// so the wizard tower panel and the in-world spell hit flash share one table.

// Graph node colors
pub(crate) const GRAPH_NODE_BG: Color = Color::srgb(0.12, 0.12, 0.15);
pub(crate) const GRAPH_NODE_BORDER: Color = Color::srgb(0.3, 0.3, 0.35);
pub(crate) const GRAPH_NODE_LOCKED_BG: Color = Color::srgb(0.08, 0.08, 0.08);
pub(crate) const GRAPH_NODE_LOCKED_BORDER: Color = Color::srgb(0.2, 0.2, 0.2);
pub(crate) const GRAPH_NODE_SELECTED_BORDER: Color = Color::srgb(0.9, 0.8, 0.2);
pub(crate) const GRAPH_NODE_COMPLETED_BORDER: Color = Color::srgb(0.3, 0.7, 0.3);
pub(crate) const GRAPH_NODE_FREE_BORDER: Color = Color::srgb(0.5, 0.5, 0.6);

// Graph edge colors
pub(crate) const GRAPH_EDGE_COLOR: Color = Color::srgba(0.4, 0.4, 0.5, 0.7);
pub(crate) const GRAPH_EDGE_LOCKED_COLOR: Color = Color::hsla(25.0, 0.10, 0.15, 0.4);

// Graph area
pub(crate) const GRAPH_AREA_BG: Color = Color::srgba(0.04, 0.04, 0.06, 0.8);

// ---------------------------------------------------------------------------
// Detail panel
// ---------------------------------------------------------------------------
// Progress fill color (used in node rings and unified slider)
pub(crate) const PROGRESS_BAR_FILL: Color = Color::srgb(0.3, 0.6, 0.9);

// Allocation slider (used in detail panel)
pub(crate) const SLIDER_TRACK_BG: Color = Color::srgb(0.2, 0.2, 0.2);
pub(crate) const SLIDER_TRACK_BORDER: Color = Color::srgb(0.35, 0.35, 0.4);
pub(crate) const SLIDER_FILL_COLOR: Color = Color::srgb(0.5, 0.7, 0.3);
pub(crate) const SLIDER_HANDLE_COLOR: Color = Color::WHITE;

// ---------------------------------------------------------------------------
// Talent UI
// ---------------------------------------------------------------------------

// Talent progress bar
pub(crate) const TALENT_BAR_BG: Color = Color::srgb(0.12, 0.12, 0.12);
pub(crate) const TALENT_BAR_FILL: Color = Color::srgb(0.6, 0.4, 0.9);

// Talent cards
/// Purple accent for selected/active talents (matches insight constellation theme).
pub(crate) const TALENT_ACTIVE_BG: Color = Color::srgb(0.18, 0.14, 0.25);
pub(crate) const TALENT_ACTIVE_BORDER: Color = Color::srgb(0.6, 0.4, 0.9);
pub(crate) const TALENT_UNLOCKED_BG: Color = Color::srgb(0.14, 0.14, 0.18);
pub(crate) const TALENT_LOCKED_BG: Color = Color::srgb(0.07, 0.07, 0.07);
pub(crate) const TALENT_LOCKED_BORDER: Color = Color::srgb(0.18, 0.18, 0.18);
pub(crate) const TALENT_UNLOCKED_BORDER: Color = Color::srgb(0.3, 0.3, 0.35);

// ---------------------------------------------------------------------------
// Insight constellation
// ---------------------------------------------------------------------------
pub(crate) const INSIGHT_NODE_BG: Color = Color::srgb(0.1, 0.08, 0.15);
pub(crate) const INSIGHT_NODE_BORDER: Color = Color::srgb(0.6, 0.4, 0.9);
pub(crate) const INSIGHT_NODE_MAXED_BORDER: Color = Color::srgb(1.0, 0.85, 0.3);
pub(crate) const INSIGHT_ANCHOR_BORDER: Color = Color::srgb(0.5, 0.3, 0.8);
pub(crate) const INSIGHT_EDGE_COLOR: Color = Color::srgba(0.5, 0.3, 0.7, 0.6);
pub(crate) const INSIGHT_PROGRESS_FILL: Color = Color::srgb(0.6, 0.4, 0.9);

// ---------------------------------------------------------------------------
// Time Travel
// ---------------------------------------------------------------------------
pub(crate) const TIME_TRAVEL_SECTION_BG: Color = Color::srgb(0.06, 0.04, 0.1);
pub(crate) const TIME_TRAVEL_SECTION_BORDER: Color = Color::srgb(0.3, 0.2, 0.5);
pub(crate) const TIME_TRAVEL_BOSS_COLOR: Color = Color::srgb(1.0, 0.7, 0.2);
pub(crate) const TIME_TRAVEL_HOVER_BG: Color = Color::srgba(1.0, 0.95, 0.7, 0.1);
pub(crate) const TIME_TRAVEL_SELECTED_BG: Color = Color::srgba(1.0, 0.85, 0.3, 0.15);
pub(crate) const TIME_TRAVEL_SELECTED_TEXT: Color = Color::srgb(1.0, 0.85, 0.3);
