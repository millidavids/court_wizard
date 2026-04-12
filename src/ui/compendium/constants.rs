use bevy::prelude::*;

use crate::ui::constants::{
    INSIGHT_COLOR as GLOBAL_INSIGHT, TEXT_BODY, TEXT_DISABLED, TEXT_MUTED, TEXT_PRIMARY,
};

// Re-export shared tab constants so `use super::constants::*` picks them up.
pub(super) use crate::ui::constants::{
    ACTIVE_TAB_BG, ACTIVE_TAB_BORDER, INACTIVE_TAB_BG, TAB_BORDER, TAB_FONT_SIZE, TAB_HEIGHT,
    TAB_PADDING_H,
};

// Colors
pub(super) const TEXT_COLOR: Color = TEXT_PRIMARY;
pub(super) const UNLOCKED_COLOR: Color = TEXT_BODY;
pub(super) const LOCKED_COLOR: Color = TEXT_DISABLED;
pub(super) const DESCRIPTION_COLOR: Color = TEXT_MUTED;
pub(super) const INSIGHT_COLOR: Color = GLOBAL_INSIGHT;
pub(super) const IN_PROGRESS_COLOR: Color = Color::srgb(0.5, 0.65, 0.9);
pub(super) const SECTION_BG: Color = Color::hsla(20.0, 0.10, 0.08, 0.8);
pub(super) const DETAIL_BG: Color = Color::hsla(20.0, 0.08, 0.06, 0.9);
pub(super) const DETAIL_BORDER: Color = Color::hsla(42.0, 0.45, 0.30, 0.8);
pub(super) const ITEM_BG: Color = Color::hsla(22.0, 0.10, 0.09, 0.75);
pub(super) const ITEM_BORDER: Color = Color::hsla(270.0, 0.15, 0.18, 0.6);
pub(super) const TEAM_DEFENDER_COLOR: Color = Color::srgb(0.3, 0.6, 0.9);
pub(super) const TEAM_ATTACKER_COLOR: Color = Color::srgb(0.9, 0.4, 0.3);
pub(super) const TEAM_BOSS_COLOR: Color = Color::srgb(0.8, 0.3, 0.8);

// Font sizes
pub(super) const TITLE_FONT_SIZE: f32 = 40.0;
pub(super) const ITEM_NAME_FONT_SIZE: f32 = 15.0;
pub(super) const DETAIL_NAME_FONT_SIZE: f32 = 24.0;
pub(super) const DETAIL_CATEGORY_FONT_SIZE: f32 = 14.0;
pub(super) const DETAIL_DESC_FONT_SIZE: f32 = 14.0;
pub(super) const DETAIL_FLAVOR_FONT_SIZE: f32 = 13.0;
pub(super) const STAT_VALUE_FONT_SIZE: f32 = 22.0;
pub(super) const STAT_LABEL_COLOR: Color = TEXT_MUTED;
pub(super) const STAT_SECTION_FONT_SIZE: f32 = 17.0;
pub(super) const STAT_SECTION_COLOR: Color = Color::hsla(0.0, 0.0, 0.95, 1.0);

// Layout
pub(super) const MARGIN_SMALL: f32 = 8.0;
pub(super) const SECTION_PADDING: f32 = 12.0;
pub(super) const LEFT_PANEL_PERCENT: f32 = 33.33;
pub(super) const RIGHT_PANEL_PERCENT: f32 = 66.67;
pub(super) const COLUMN_GAP: f32 = 12.0;
pub(super) const DETAIL_ICON_SIZE: f32 = 64.0;
