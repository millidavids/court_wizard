//! WizardSelect-phase panels.
//!
//! LEFT  = match details: both players, their wizard picks, ready status,
//!         plus Ready/Unready, host-only Start Game, and Disconnect.
//! RIGHT = this player's selected wizard + a Switch Wizard button (the shared
//!         wizard-card grid). Switch Wizard is disabled while you are Ready.

use bevy::prelude::*;

use crate::config::WizardType;
use crate::networking::resources::{NetworkConnection, PeerRole};
use crate::ui::constants::{SUCCESS_COLOR, TEXT_MUTED, TEXT_PRIMARY, WARNING_COLOR};
use crate::ui::systems::spawn_button;

use super::panel_styles::{
    BODY_FONT_SIZE, BUTTON_STYLE, DISABLED_BUTTON_STYLE, DISCONNECT_BUTTON_STYLE, HEADING_FONT_SIZE,
    READY_BUTTON_STYLE, SECTION_FONT_SIZE, UNREADY_BUTTON_STYLE,
};
use super::panels::spawn_ping_row;
use super::state::MpTabAction;

/// Left panel — match details and the Ready / Start Game / Disconnect buttons.
#[allow(clippy::too_many_arguments)]
pub(super) fn build_wizard_select_left(
    commands: &mut Commands,
    entity: Entity,
    my_wizard: Option<WizardType>,
    opponent_wizard: Option<WizardType>,
    my_ready: bool,
    opponent_ready: bool,
    connection: &NetworkConnection,
) {
    let is_host = connection.role == Some(PeerRole::Host);
    let both_ready = my_ready && opponent_ready;

    commands.entity(entity).with_children(|left| {
        left.spawn((
            Text::new("Multiplayer Match"),
            TextFont::from_font_size(HEADING_FONT_SIZE),
            TextColor(TEXT_PRIMARY),
            Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
        ));

        spawn_player_row(left, "You", my_wizard, my_ready);
        spawn_divider(left);
        spawn_player_row(left, "Opponent", opponent_wizard, opponent_ready);

        if let Some(ping) = connection.ping_ms {
            spawn_ping_row(left, ping);
        }

        // Spacer pushes the buttons toward the bottom.
        left.spawn(Node {
            flex_grow: 1.0,
            min_height: Val::Px(12.0),
            ..default()
        });

        if my_ready {
            spawn_button(left, "Unready", MpTabAction::Unready, &UNREADY_BUTTON_STYLE);
        } else {
            spawn_button(left, "Ready!", MpTabAction::Ready, &READY_BUTTON_STYLE);
        }

        if is_host {
            if both_ready {
                spawn_button(left, "Start Game", MpTabAction::StartGame, &BUTTON_STYLE);
            } else {
                // Disabled until both players are ready — empty `()` action.
                spawn_button(left, "Start Game", (), &DISABLED_BUTTON_STYLE);
            }
        } else if both_ready {
            left.spawn((
                Text::new("Waiting for host to start..."),
                TextFont::from_font_size(BODY_FONT_SIZE),
                TextColor(SUCCESS_COLOR),
                Node {
                    margin: UiRect::vertical(Val::Px(6.0)),
                    ..default()
                },
            ));
        }

        spawn_button(
            left,
            "Disconnect",
            MpTabAction::Disconnect,
            &DISCONNECT_BUTTON_STYLE,
        );
    });
}

/// Right panel — this player's selected wizard and the Switch Wizard button.
pub(super) fn build_wizard_select_right(
    commands: &mut Commands,
    entity: Entity,
    my_wizard: Option<WizardType>,
    my_ready: bool,
) {
    commands.entity(entity).with_children(|right| {
        right.spawn((
            Text::new("Your Wizard"),
            TextFont::from_font_size(SECTION_FONT_SIZE),
            TextColor(TEXT_MUTED),
        ));
        right.spawn((
            Text::new(
                my_wizard
                    .map(|w| w.display_name().to_string())
                    .unwrap_or_else(|| "Not selected".to_string()),
            ),
            TextFont::from_font_size(HEADING_FONT_SIZE),
            TextColor(TEXT_PRIMARY),
        ));
        if let Some(w) = my_wizard {
            right.spawn((
                Text::new(w.description().to_string()),
                TextFont::from_font_size(BODY_FONT_SIZE),
                TextColor(TEXT_MUTED),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ));
        }

        if my_ready {
            // Can't change wizard while Ready — unready first.
            spawn_button(right, "Switch Wizard", (), &DISABLED_BUTTON_STYLE);
            right.spawn((
                Text::new("Unready to switch wizard."),
                TextFont::from_font_size(BODY_FONT_SIZE),
                TextColor(TEXT_MUTED),
            ));
        } else {
            spawn_button(
                right,
                "Switch Wizard",
                MpTabAction::SwitchWizard,
                &BUTTON_STYLE,
            );
        }
    });
}

fn spawn_player_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    wizard: Option<WizardType>,
    ready: bool,
) {
    parent.spawn((
        Text::new(label),
        TextFont::from_font_size(SECTION_FONT_SIZE),
        TextColor(TEXT_MUTED),
    ));
    parent.spawn((
        Text::new(
            wizard
                .map(|w| w.display_name().to_string())
                .unwrap_or_else(|| "Choosing...".to_string()),
        ),
        TextFont::from_font_size(HEADING_FONT_SIZE),
        TextColor(TEXT_PRIMARY),
    ));
    let (status_text, status_color) = if ready {
        ("Ready!", SUCCESS_COLOR)
    } else {
        ("Not ready", WARNING_COLOR)
    };
    parent.spawn((
        Text::new(status_text),
        TextFont::from_font_size(BODY_FONT_SIZE),
        TextColor(status_color),
    ));
}

fn spawn_divider(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Percent(80.0),
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(Color::hsla(0.0, 0.0, 0.25, 0.5)),
    ));
}
