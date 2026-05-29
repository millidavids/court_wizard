//! Connect-phase right panel: Steam Invite (if Steam is up) + Host / Join
//! code-share fallback + relay toggle + last error.

use bevy::prelude::*;

use crate::networking::resources::NetworkConnection;
use crate::ui::constants::{ERROR_COLOR, TEXT_MUTED, TEXT_PRIMARY};
use crate::ui::systems::spawn_button;

use super::panel_styles::{
    BODY_FONT_SIZE, BUTTON_STYLE, HEADING_FONT_SIZE, HINT_FONT_SIZE, SMALL_BUTTON_STYLE,
};
use super::state::MpTabAction;

pub(super) fn build_connect(
    commands: &mut Commands,
    entity: Entity,
    connection: &NetworkConnection,
    use_relay: bool,
    steam_available: bool,
) {
    commands.entity(entity).with_children(|right| {
        right.spawn((
            Text::new("Multiplayer"),
            TextFont::from_font_size(HEADING_FONT_SIZE),
            TextColor(TEXT_PRIMARY),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ));

        let intro = if steam_available {
            "Invite a friend through Steam, or share a code with anyone."
        } else {
            "Host a game and share your code, or join a friend's game with their code."
        };
        right.spawn((
            Text::new(intro),
            TextFont::from_font_size(BODY_FONT_SIZE),
            TextColor(TEXT_MUTED),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ));

        if steam_available {
            spawn_button(
                right,
                "Invite Friend on Steam",
                MpTabAction::SteamInvite,
                &BUTTON_STYLE,
            );
            right.spawn((
                Text::new("— or share a code —"),
                TextFont::from_font_size(HINT_FONT_SIZE),
                TextColor(TEXT_MUTED),
                Node {
                    margin: UiRect::vertical(Val::Px(6.0)),
                    ..default()
                },
            ));
        }

        spawn_button(right, "Host Game", MpTabAction::HostGame, &BUTTON_STYLE);
        spawn_button(right, "Join Game", MpTabAction::JoinGame, &BUTTON_STYLE);

        let relay_label = if use_relay {
            "Mode: Online"
        } else {
            "Mode: LAN"
        };
        let relay_hint = if use_relay {
            "Online — play with friends anywhere."
        } else {
            "LAN — same home network only."
        };
        spawn_button(
            right,
            relay_label,
            MpTabAction::ToggleRelay,
            &SMALL_BUTTON_STYLE,
        );

        right.spawn((
            Text::new(relay_hint),
            TextFont::from_font_size(HINT_FONT_SIZE),
            TextColor(TEXT_MUTED),
        ));

        if let Some(err) = &connection.error {
            right.spawn((
                Text::new(format!("Last error: {}", err)),
                TextFont::from_font_size(HINT_FONT_SIZE),
                TextColor(ERROR_COLOR),
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
            ));
        }
    });
}
