//! Manual screen constants.

use bevy::prelude::*;

use crate::ui::components::ButtonStyle;
use crate::ui::constants::{BACK_BUTTON_STYLE, TEXT_PRIMARY};

pub(super) const TITLE_FONT_SIZE: f32 = 48.0;
pub(super) const TEXT_COLOR: Color = TEXT_PRIMARY;

pub(super) const CHANGELOG_WEBSITE_URL: &str = "https://courtwizard.blackhearthgames.com/changelog";

pub(super) const RECENT_RELEASES_COUNT: usize = 5;

pub(super) const CHANGELOG_LINK_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 320.0,
    ..BACK_BUTTON_STYLE
};
