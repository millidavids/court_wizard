//! Hosting-phase right panel: show local ticket code, wait for guest.

use bevy::prelude::*;

use crate::networking::resources::NetworkConnection;
use crate::ui::constants::{TEXT_MUTED, TEXT_PRIMARY, WARNING_COLOR};
use crate::ui::systems::spawn_button;

use super::panel_styles::{
    BODY_FONT_SIZE, CODE_BOX_BG, CODE_BOX_BORDER_UNFOCUSED, CODE_FONT_SIZE, HEADING_FONT_SIZE,
    INLINE_BUTTON_STYLE, SECTION_FONT_SIZE, SMALL_BUTTON_STYLE,
};
use super::panels::spawn_ping_row;
use super::state::MpTabAction;

pub(super) fn build_hosting(
    commands: &mut Commands,
    entity: Entity,
    connection: &NetworkConnection,
) {
    commands.entity(entity).with_children(|right| {
        right.spawn((
            Text::new("Hosting"),
            TextFont::from_font_size(HEADING_FONT_SIZE),
            TextColor(TEXT_PRIMARY),
        ));

        right.spawn((
            Text::new("Share this code with your friend:"),
            TextFont::from_font_size(BODY_FONT_SIZE),
            TextColor(TEXT_MUTED),
        ));

        if let Some(code) = &connection.local_code {
            // Row: [Copy button] [code box] — keeps it clear what is being copied.
            right
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    margin: UiRect::vertical(Val::Px(8.0)),
                    ..default()
                })
                .with_children(|row| {
                    // Fixed-shrink slot so the button keeps its width and
                    // padding when the code box wants to grow.
                    row.spawn(Node {
                        flex_shrink: 0.0,
                        ..default()
                    })
                    .with_children(|slot| {
                        spawn_button(slot, "Copy", MpTabAction::CopyCode, &INLINE_BUTTON_STYLE);
                    });

                    row.spawn((
                        Node {
                            flex_grow: 1.0,
                            padding: UiRect::all(Val::Px(8.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            flex_wrap: FlexWrap::Wrap,
                            ..default()
                        },
                        BackgroundColor(CODE_BOX_BG),
                        BorderColor::all(CODE_BOX_BORDER_UNFOCUSED),
                    ))
                    .with_children(|box_inner| {
                        box_inner.spawn((
                            Text::new(code.clone()),
                            TextFont::from_font_size(CODE_FONT_SIZE),
                            TextColor(TEXT_PRIMARY),
                            TextLayout::new_with_linebreak(LineBreak::AnyCharacter),
                            Node {
                                max_width: Val::Percent(100.0),
                                ..default()
                            },
                        ));
                    });
                });
        } else {
            right.spawn((
                Text::new("Generating code..."),
                TextFont::from_font_size(BODY_FONT_SIZE),
                TextColor(WARNING_COLOR),
                Node {
                    margin: UiRect::vertical(Val::Px(8.0)),
                    ..default()
                },
            ));
        }

        right.spawn((
            Text::new("Waiting for opponent to connect..."),
            TextFont::from_font_size(SECTION_FONT_SIZE),
            TextColor(WARNING_COLOR),
        ));

        if let Some(ping) = connection.ping_ms {
            spawn_ping_row(right, ping);
        }

        spawn_button(right, "Cancel", MpTabAction::Cancel, &SMALL_BUTTON_STYLE);
    });
}
