use bevy::prelude::*;

use crate::ui::components::ButtonStyle;
use crate::ui::constants::{
    BUTTON_BG, BUTTON_BORDER, INSIGHT_COLOR as GLOBAL_INSIGHT, TEXT_BODY, TEXT_PRIMARY,
};

pub const TITLE_COLOR: Color = TEXT_PRIMARY;
pub const TEXT_COLOR: Color = TEXT_BODY;
pub const INSIGHT_COLOR: Color = GLOBAL_INSIGHT;
pub const CARNAGE_MET_COLOR: Color = Color::srgb(0.8, 0.1, 0.1);
pub const CARNAGE_UNMET_COLOR: Color = Color::srgb(0.6, 0.4, 0.1);

pub const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 250.0,
    height: 65.0,
    border_width: 3.0,
    font_size: 20.0,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};
