//! Guest-side Multiplayer-tab LEFT panel: a live mirror of the host's selected
//! game mode (from `CoopHostSelection`), the guest's own wizard/ready status, and
//! the Ready/Unready + Disconnect buttons pinned at the bottom. The guest does
//! everything from this one screen — the host alone picks the mode.

use bevy::prelude::*;

use crate::config::WizardType;
use crate::networking::protocol::HostMode;
use crate::networking::resources::NetworkConnection;
use crate::ui::constants::{TEXT_MUTED, TEXT_PRIMARY};
use crate::ui::systems::spawn_button;

use super::panel_styles::{
    BODY_FONT_SIZE, DISCONNECT_BUTTON_STYLE, HEADING_FONT_SIZE, READY_BUTTON_STYLE,
    SECTION_FONT_SIZE, UNREADY_BUTTON_STYLE,
};
use super::panel_wizard_select::spawn_player_row;
use super::panels::spawn_ping_row;
use super::state::{CoopHostSelection, MpTabAction};

/// Builds the guest's left panel: host-mode mirror + the guest's own status +
/// pinned Ready/Disconnect. `my_wizard`/`my_ready` are the guest's own selection.
pub(super) fn build_guest_mode_mirror(
    commands: &mut Commands,
    entity: Entity,
    host_selection: Option<&CoopHostSelection>,
    my_wizard: Option<WizardType>,
    my_ready: bool,
    connection: &NetworkConnection,
) {
    // `CoopHostSelection` is always present (init_resource'd); fall back to the
    // default (Browsing) if a caller ever passes None.
    let fallback = CoopHostSelection::default();
    let sel = host_selection.unwrap_or(&fallback);

    commands.entity(entity).with_children(|left| {
        // ----- Host's selected mode (the "detailed mirror") -----
        let title = match sel.mode {
            HostMode::Endless => {
                let suffix = if sel.is_continue {
                    "Continue"
                } else {
                    "New Game"
                };
                format!("Endless — Level {} ({suffix})", sel.level)
            }
            HostMode::Roguelite => {
                let suffix = if sel.is_continue {
                    "Continue Run"
                } else {
                    "New Run"
                };
                format!("Roguelite — Level {} ({suffix})", sel.level)
            }
            HostMode::Versus => "Versus Duel".to_string(),
            HostMode::Browsing => "Host is choosing a mode...".to_string(),
        };
        left.spawn((
            Text::new(title),
            TextFont::from_font_size(HEADING_FONT_SIZE),
            TextColor(TEXT_PRIMARY),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ));

        if let Some(w) = sel.host_wizard {
            left.spawn((
                Text::new(format!("Host's wizard: {}", w.display_name())),
                TextFont::from_font_size(BODY_FONT_SIZE),
                TextColor(TEXT_MUTED),
            ));
        }
        for line in &sel.detail_lines {
            left.spawn((
                Text::new(line.clone()),
                TextFont::from_font_size(BODY_FONT_SIZE),
                TextColor(TEXT_MUTED),
            ));
        }
        if sel.mode != HostMode::Browsing {
            left.spawn((
                Text::new("The host picks the mode."),
                TextFont::from_font_size(SECTION_FONT_SIZE),
                TextColor(TEXT_MUTED),
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
            ));
        }

        // ----- The guest's own wizard + ready status -----
        left.spawn(Node {
            height: Val::Px(12.0),
            ..default()
        });
        spawn_player_row(left, "You", my_wizard, my_ready);

        if let Some(ping) = connection.ping_ms {
            spawn_ping_row(left, ping);
        }

        // Spacer pushes the buttons to the bottom.
        left.spawn(Node {
            flex_grow: 1.0,
            min_height: Val::Px(12.0),
            ..default()
        });

        // ----- Pinned bottom: Ready/Unready + Disconnect -----
        if my_ready {
            spawn_button(left, "Unready", MpTabAction::Unready, &UNREADY_BUTTON_STYLE);
        } else {
            spawn_button(left, "Ready!", MpTabAction::Ready, &READY_BUTTON_STYLE);
        }
        spawn_button(
            left,
            "Disconnect",
            MpTabAction::Disconnect,
            &DISCONNECT_BUTTON_STYLE,
        );
    });
}
