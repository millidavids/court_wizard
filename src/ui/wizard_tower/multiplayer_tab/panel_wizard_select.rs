//! Wizard-select phase panel builders: pick wizard, ready up, see opponent.

use bevy::prelude::*;

use crate::config::WizardType;
use crate::networking::resources::NetworkConnection;
use crate::ui::constants::{SUCCESS_COLOR, TEXT_MUTED, TEXT_PRIMARY, WARNING_COLOR};
use crate::ui::systems::spawn_button;

use super::panel_styles::{
    BODY_FONT_SIZE, CARD_BG, CARD_BORDER, CARD_BORDER_RADIUS, CARD_BORDER_SELECTED,
    CARD_BORDER_WIDTH, DISCONNECT_BUTTON_STYLE, HEADING_FONT_SIZE, HINT_FONT_SIZE,
    READY_BUTTON_STYLE, SECTION_FONT_SIZE, UNREADY_BUTTON_STYLE,
};
use super::panels::spawn_ping_row;
use super::state::{MpTabAction, MpWizardCardMarker};

pub(super) fn build_wizard_select_left(
    commands: &mut Commands,
    entity: Entity,
    my_wizard_types: &[WizardType],
    my_wizard: Option<WizardType>,
    my_ready: bool,
) {
    commands.entity(entity).with_children(|left| {
        left.spawn((
            Text::new("Pick Your Wizard"),
            TextFont::from_font_size(HEADING_FONT_SIZE),
            TextColor(TEXT_PRIMARY),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ));

        for &wizard_type in my_wizard_types {
            let is_selected = my_wizard == Some(wizard_type);
            let border_color = if is_selected {
                CARD_BORDER_SELECTED
            } else {
                CARD_BORDER
            };

            left.spawn((
                Button,
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(Val::Px(CARD_BORDER_WIDTH)),
                    padding: UiRect::all(Val::Px(8.0)),
                    row_gap: Val::Px(2.0),
                    margin: UiRect::bottom(Val::Px(4.0)),
                    border_radius: BorderRadius::all(Val::Px(CARD_BORDER_RADIUS)),
                    ..default()
                },
                BackgroundColor(CARD_BG),
                BorderColor::all(border_color),
                crate::ui::components::ButtonColors {
                    background: CARD_BG,
                    border: border_color,
                },
                MpTabAction::SelectWizard(wizard_type),
                MpWizardCardMarker(wizard_type),
                crate::ui::focus::Focusable,
            ))
            .with_children(|card| {
                card.spawn((
                    Text::new(wizard_type.display_name()),
                    TextFont::from_font_size(SECTION_FONT_SIZE),
                    TextColor(TEXT_PRIMARY),
                ));
                card.spawn((
                    Text::new(wizard_type.locked_description()),
                    TextFont::from_font_size(HINT_FONT_SIZE),
                    TextColor(TEXT_MUTED),
                ));
            });
        }

        left.spawn((Node {
            margin: UiRect::top(Val::Px(12.0)),
            ..default()
        },))
            .with_children(|area| {
                if my_ready {
                    spawn_button(
                        area,
                        "Unready",
                        MpTabAction::Unready,
                        &UNREADY_BUTTON_STYLE,
                    );
                } else {
                    spawn_button(area, "Ready!", MpTabAction::Ready, &READY_BUTTON_STYLE);
                }
            });

        left.spawn(Node {
            flex_grow: 1.0,
            ..default()
        });
        spawn_button(
            left,
            "Disconnect",
            MpTabAction::Disconnect,
            &DISCONNECT_BUTTON_STYLE,
        );
    });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_wizard_select_right(
    commands: &mut Commands,
    entity: Entity,
    my_wizard: Option<WizardType>,
    opponent_wizard: Option<WizardType>,
    my_ready: bool,
    opponent_ready: bool,
    connection: &NetworkConnection,
) {
    commands.entity(entity).with_children(|right| {
        right.spawn((
            Text::new("Your Selection"),
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
            Node {
                margin: UiRect::bottom(Val::Px(12.0)),
                ..default()
            },
        ));

        let my_status_color = if my_ready { SUCCESS_COLOR } else { TEXT_MUTED };
        let my_status_text = if my_ready { "Ready!" } else { "Not ready" };
        right.spawn((
            Text::new(format!("You: {}", my_status_text)),
            TextFont::from_font_size(BODY_FONT_SIZE),
            TextColor(my_status_color),
        ));

        right.spawn((
            Node {
                width: Val::Percent(80.0),
                height: Val::Px(1.0),
                margin: UiRect::vertical(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::hsla(0.0, 0.0, 0.25, 0.5)),
        ));

        right.spawn((
            Text::new("Opponent"),
            TextFont::from_font_size(SECTION_FONT_SIZE),
            TextColor(TEXT_MUTED),
        ));
        right.spawn((
            Text::new(
                opponent_wizard
                    .map(|w| w.display_name().to_string())
                    .unwrap_or_else(|| "Choosing...".to_string()),
            ),
            TextFont::from_font_size(HEADING_FONT_SIZE),
            TextColor(TEXT_PRIMARY),
            Node {
                margin: UiRect::bottom(Val::Px(6.0)),
                ..default()
            },
        ));

        let opp_status_color = if opponent_ready {
            SUCCESS_COLOR
        } else {
            WARNING_COLOR
        };
        let opp_status_text = if opponent_ready {
            "Ready!"
        } else {
            "Not ready"
        };
        right.spawn((
            Text::new(format!("Opponent: {}", opp_status_text)),
            TextFont::from_font_size(BODY_FONT_SIZE),
            TextColor(opp_status_color),
        ));

        if let Some(ping) = connection.ping_ms {
            spawn_ping_row(right, ping);
        }

        if my_ready && opponent_ready {
            right.spawn((
                Text::new("Both players ready — starting match!"),
                TextFont::from_font_size(BODY_FONT_SIZE),
                TextColor(SUCCESS_COLOR),
                Node {
                    margin: UiRect::top(Val::Px(12.0)),
                    ..default()
                },
            ));
        }
    });
}
