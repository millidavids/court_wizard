use bevy::prelude::*;

use crate::ui::components::ButtonStyle;

// ---------------------------------------------------------------------------
// Shared layout
// ---------------------------------------------------------------------------
pub(super) const BACKGROUND_COLOR: Color = Color::srgb(0.08, 0.08, 0.1);
pub(super) const TITLE_FONT_SIZE: f32 = 36.0;
pub(super) const LEVEL_FONT_SIZE: f32 = 18.0;
pub(super) const INSIGHT_FONT_SIZE: f32 = 22.0;

// ---------------------------------------------------------------------------
// Shared colors
// ---------------------------------------------------------------------------
pub(super) const TITLE_COLOR: Color = Color::srgb(0.95, 0.95, 0.95);
pub(super) const TEXT_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);
pub(super) const INSIGHT_COLOR: Color = Color::srgb(0.6, 0.8, 1.0);
pub(super) const LOCKED_TEXT_COLOR: Color = Color::srgb(0.45, 0.45, 0.45);
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

// ---------------------------------------------------------------------------
// Graph layout
// ---------------------------------------------------------------------------
pub(super) const GRAPH_NODE_SIZE: f32 = 60.0;
pub(super) const GRAPH_EDGE_THICKNESS: f32 = 2.0;
pub(super) const GRAPH_FREE_NODE_SIZE: f32 = 40.0;

// Graph node colors
pub(super) const GRAPH_NODE_BG: Color = Color::srgba(0.12, 0.12, 0.15, 0.95);
pub(super) const GRAPH_NODE_BORDER: Color = Color::srgb(0.3, 0.3, 0.35);
pub(super) const GRAPH_NODE_LOCKED_BG: Color = Color::srgba(0.08, 0.08, 0.08, 0.9);
pub(super) const GRAPH_NODE_LOCKED_BORDER: Color = Color::srgb(0.2, 0.2, 0.2);
pub(super) const GRAPH_NODE_SELECTED_BORDER: Color = Color::srgb(0.9, 0.8, 0.2);
pub(super) const GRAPH_NODE_COMPLETED_BORDER: Color = Color::srgb(0.3, 0.7, 0.3);
pub(super) const GRAPH_NODE_FREE_BORDER: Color = Color::srgb(0.5, 0.5, 0.6);

// Graph edge colors
pub(super) const GRAPH_EDGE_COLOR: Color = Color::srgba(0.4, 0.4, 0.5, 0.7);
pub(super) const GRAPH_EDGE_LOCKED_COLOR: Color = Color::srgba(0.2, 0.2, 0.25, 0.4);

// Graph zoom
pub(super) const GRAPH_ZOOM_MIN: f32 = 0.4;
pub(super) const GRAPH_ZOOM_MAX: f32 = 2.0;
pub(super) const GRAPH_ZOOM_SPEED: f32 = 0.1;

// Graph area
pub(super) const GRAPH_AREA_BG: Color = Color::srgba(0.04, 0.04, 0.06, 0.8);

// ---------------------------------------------------------------------------
// Detail panel
// ---------------------------------------------------------------------------
pub(super) const DETAIL_PANEL_WIDTH: f32 = 280.0;
pub(super) const DETAIL_PANEL_PADDING: f32 = 16.0;
pub(super) const DETAIL_PANEL_BG: Color = Color::srgba(0.1, 0.1, 0.13, 0.95);
pub(super) const DETAIL_PANEL_BORDER: Color = Color::srgb(0.3, 0.3, 0.4);
pub(super) const DETAIL_TITLE_FONT_SIZE: f32 = 18.0;
pub(super) const DETAIL_TEXT_FONT_SIZE: f32 = 13.0;
pub(super) const DETAIL_SMALL_FONT_SIZE: f32 = 11.0;

// Progress bar (shows existing research progress)
pub(super) const PROGRESS_BAR_HEIGHT: f32 = 8.0;
pub(super) const PROGRESS_BAR_BACKGROUND: Color = Color::srgb(0.15, 0.15, 0.15);
pub(super) const PROGRESS_BAR_FILL: Color = Color::srgb(0.3, 0.6, 0.9);
pub(super) const PROGRESS_BAR_FULL: Color = Color::srgb(0.4, 0.9, 0.4);

// Allocation slider (used in detail panel)
pub(super) const SLIDER_TRACK_WIDTH: f32 = 220.0;
pub(super) const SLIDER_TRACK_HEIGHT: f32 = 12.0;
pub(super) const SLIDER_HANDLE_WIDTH: f32 = 6.0;
pub(super) const SLIDER_HANDLE_HEIGHT: f32 = 22.0;
pub(super) const SLIDER_TRACK_BG: Color = Color::srgb(0.2, 0.2, 0.2);
pub(super) const SLIDER_TRACK_BORDER: Color = Color::srgb(0.35, 0.35, 0.4);
pub(super) const SLIDER_FILL_COLOR: Color = Color::srgb(0.5, 0.7, 0.3);
pub(super) const SLIDER_HANDLE_COLOR: Color = Color::WHITE;

// Spell icon
pub(super) const SPELL_ICON_SIZE: f32 = 32.0;

// ---------------------------------------------------------------------------
// Buttons
// ---------------------------------------------------------------------------
pub(super) const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 220.0,
    height: 55.0,
    border_width: 3.0,
    font_size: 16.0,
    background: Color::hsla(0.0, 0.0, 0.15, 1.0),
    border: Color::hsla(0.0, 0.0, 0.3, 1.0),
    text_color: Color::hsla(0.0, 0.0, 0.9, 1.0),
};

pub(super) const COMMIT_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 180.0,
    height: 50.0,
    border_width: 3.0,
    font_size: 16.0,
    background: Color::srgb(0.15, 0.3, 0.15),
    border: Color::srgb(0.3, 0.6, 0.3),
    text_color: Color::srgb(0.85, 1.0, 0.85),
};

pub(super) const BACK_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 140.0,
    height: 50.0,
    border_width: 3.0,
    font_size: 16.0,
    background: Color::hsla(0.0, 0.0, 0.15, 1.0),
    border: Color::hsla(0.0, 0.0, 0.3, 1.0),
    text_color: Color::hsla(0.0, 0.0, 0.9, 1.0),
};

#[cfg(debug_assertions)]
pub(super) const DEBUG_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 200.0,
    height: 50.0,
    border_width: 3.0,
    font_size: 14.0,
    background: Color::srgb(0.4, 0.1, 0.1),
    border: Color::srgb(0.8, 0.2, 0.2),
    text_color: Color::srgb(1.0, 0.7, 0.7),
};
