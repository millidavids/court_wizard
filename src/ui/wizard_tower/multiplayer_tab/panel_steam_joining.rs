//! Steam-joining right panel: "Connecting via Steam…" + Cancel.

use bevy::prelude::*;

use crate::ui::constants::{TEXT_MUTED, TEXT_PRIMARY, WARNING_COLOR};
use crate::ui::systems::spawn_button;

use super::panel_styles::{
    BODY_FONT_SIZE, HEADING_FONT_SIZE, SECTION_FONT_SIZE, SMALL_BUTTON_STYLE,
};
use super::state::MpTabAction;

pub(super) fn build_steam_joining(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).with_children(|right| {
        right.spawn((
            Text::new("Joining via Steam"),
            TextFont::from_font_size(HEADING_FONT_SIZE),
            TextColor(TEXT_PRIMARY),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ));

        right.spawn((
            Text::new("Hooking up to your friend's Steam lobby."),
            TextFont::from_font_size(BODY_FONT_SIZE),
            TextColor(TEXT_MUTED),
            Node {
                margin: UiRect::bottom(Val::Px(12.0)),
                ..default()
            },
        ));

        right.spawn((
            Text::new("Connecting via Steam relay…"),
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
