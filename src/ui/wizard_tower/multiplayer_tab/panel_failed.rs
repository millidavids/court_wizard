//! Failed-phase right panel: connection error, retry/back.

use bevy::prelude::*;

use crate::ui::constants::{ERROR_COLOR, TEXT_MUTED};
use crate::ui::systems::spawn_button;

use super::panel_styles::{BODY_FONT_SIZE, BUTTON_STYLE, HEADING_FONT_SIZE, SMALL_BUTTON_STYLE};
use super::state::MpTabAction;

pub(super) fn build_failed(commands: &mut Commands, entity: Entity, reason: &str) {
    commands.entity(entity).with_children(|right| {
        right.spawn((
            Text::new("Connection Failed"),
            TextFont::from_font_size(HEADING_FONT_SIZE),
            TextColor(ERROR_COLOR),
        ));
        right.spawn((
            Text::new(reason.to_string()),
            TextFont::from_font_size(BODY_FONT_SIZE),
            TextColor(TEXT_MUTED),
            Node {
                margin: UiRect::vertical(Val::Px(8.0)),
                ..default()
            },
        ));

        spawn_button(right, "Try Again", MpTabAction::Retry, &BUTTON_STYLE);
        spawn_button(right, "Back", MpTabAction::Cancel, &SMALL_BUTTON_STYLE);
    });
}
