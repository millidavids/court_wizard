//! Failed-phase panel builders: connection error, retry/back.

use bevy::prelude::*;

use crate::ui::constants::{ERROR_COLOR, TEXT_MUTED};
use crate::ui::systems::spawn_button;

use super::panel_styles::{BUTTON_STYLE, HEADING_FONT_SIZE, SMALL_BUTTON_STYLE, BODY_FONT_SIZE};
use super::state::MpTabAction;

pub(super) fn build_failed_left(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).with_children(|left| {
        spawn_button(left, "Try Again", MpTabAction::Retry, &BUTTON_STYLE);
        spawn_button(left, "Back", MpTabAction::Cancel, &SMALL_BUTTON_STYLE);
    });
}

pub(super) fn build_failed_right(commands: &mut Commands, entity: Entity, reason: &str) {
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
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },
        ));
    });
}
