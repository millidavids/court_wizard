//! Handshake-phase panel builders: connected, waiting on PlayerInfo exchange.

use bevy::prelude::*;

use crate::networking::resources::NetworkConnection;
use crate::ui::constants::{SUCCESS_COLOR, TEXT_MUTED};
use crate::ui::systems::spawn_button;

use super::panel_styles::{BODY_FONT_SIZE, HEADING_FONT_SIZE, SMALL_BUTTON_STYLE};
use super::panels::spawn_ping_row;
use super::state::MpTabAction;

pub(super) fn build_handshake_left(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).with_children(|left| {
        spawn_button(left, "Cancel", MpTabAction::Cancel, &SMALL_BUTTON_STYLE);
    });
}

pub(super) fn build_handshake_right(
    commands: &mut Commands,
    entity: Entity,
    connection: &NetworkConnection,
) {
    commands.entity(entity).with_children(|right| {
        right.spawn((
            Text::new("Connected!"),
            TextFont::from_font_size(HEADING_FONT_SIZE),
            TextColor(SUCCESS_COLOR),
        ));
        right.spawn((
            Text::new("Exchanging player info with opponent..."),
            TextFont::from_font_size(BODY_FONT_SIZE),
            TextColor(TEXT_MUTED),
        ));

        if let Some(ping) = connection.ping_ms {
            spawn_ping_row(right, ping);
        }
    });
}
