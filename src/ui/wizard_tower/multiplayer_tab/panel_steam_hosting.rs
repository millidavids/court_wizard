//! Steam-hosting right panel: "Waiting for friend to accept invite…" + Cancel.

use bevy::prelude::*;

use crate::ui::constants::{TEXT_MUTED, TEXT_PRIMARY, WARNING_COLOR};
use crate::ui::systems::spawn_button;

use super::panel_styles::{
    BODY_FONT_SIZE, HEADING_FONT_SIZE, SECTION_FONT_SIZE, SMALL_BUTTON_STYLE,
};
use super::state::MpTabAction;

pub(super) fn build_steam_hosting(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).with_children(|right| {
        right.spawn((
            Text::new("Steam Invite Sent"),
            TextFont::from_font_size(HEADING_FONT_SIZE),
            TextColor(TEXT_PRIMARY),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ));

        right.spawn((
            Text::new("Pick a friend in the Steam overlay and send them an invite."),
            TextFont::from_font_size(BODY_FONT_SIZE),
            TextColor(TEXT_MUTED),
            Node {
                margin: UiRect::bottom(Val::Px(12.0)),
                ..default()
            },
        ));

        right.spawn((
            Text::new("Waiting for your friend to accept…"),
            TextFont::from_font_size(SECTION_FONT_SIZE),
            TextColor(WARNING_COLOR),
            Node {
                margin: UiRect::bottom(Val::Px(12.0)),
                ..default()
            },
        ));

        spawn_button(right, "Cancel", MpTabAction::Cancel, &SMALL_BUTTON_STYLE);
    });
}
