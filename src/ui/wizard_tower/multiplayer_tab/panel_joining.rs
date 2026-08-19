//! Joining-phase right panel: paste code, focused text input, connect.

use bevy::prelude::*;

use crate::ui::constants::{TEXT_MUTED, TEXT_PRIMARY};
use crate::ui::systems::spawn_button;

use super::panel_styles::{
    BODY_FONT_SIZE, BUTTON_STYLE, CODE_BOX_BG, CODE_BOX_BORDER_FOCUSED, CODE_BOX_BORDER_UNFOCUSED,
    CODE_FONT_SIZE, HEADING_FONT_SIZE, HINT_FONT_SIZE, INLINE_BUTTON_STYLE, SMALL_BUTTON_STYLE,
};
use super::state::{JoinCodeInputBox, JoinCodeInputDisplay, MpTabAction, MultiplayerLobby};

pub(super) fn build_joining(commands: &mut Commands, entity: Entity, lobby: &MultiplayerLobby) {
    commands.entity(entity).with_children(|right| {
        right.spawn((
            Text::new("Join Game"),
            TextFont::from_font_size(HEADING_FONT_SIZE),
            TextColor(TEXT_PRIMARY),
        ));

        right.spawn((
            Text::new("Paste the host's connection code below, then click Connect."),
            TextFont::from_font_size(BODY_FONT_SIZE),
            TextColor(TEXT_MUTED),
        ));

        right.spawn((
            Text::new("The code stays valid while the host keeps their screen open."),
            TextFont::from_font_size(HINT_FONT_SIZE),
            TextColor(TEXT_MUTED),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ));

        let border_color = if lobby.join_code_focused {
            CODE_BOX_BORDER_FOCUSED
        } else {
            CODE_BOX_BORDER_UNFOCUSED
        };

        // Row: [Paste button] [join code input box] — keeps it clear where the
        // pasted code lands.
        right
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            })
            .with_children(|row| {
                // Fixed-shrink slot so the button keeps its width and padding
                // when the input box wants to grow.
                row.spawn(Node {
                    flex_shrink: 0.0,
                    ..default()
                })
                .with_children(|slot| {
                    spawn_button(
                        slot,
                        "Paste",
                        MpTabAction::PasteFromClipboard,
                        &INLINE_BUTTON_STYLE,
                    );
                });

                row.spawn((
                    Button,
                    Node {
                        flex_grow: 1.0,
                        min_height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(1.0)),
                        padding: UiRect::all(Val::Px(8.0)),
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        flex_wrap: FlexWrap::Wrap,
                        ..default()
                    },
                    BorderColor::all(border_color),
                    BackgroundColor(CODE_BOX_BG),
                    JoinCodeInputBox,
                    crate::ui::focus::Focusable,
                    crate::ui::focus::FocusableFlatBackground { base: CODE_BOX_BG },
                ))
                .with_children(|input| {
                    let display_text = if lobby.join_code_input.is_empty() {
                        "Click to type or paste code...".to_string()
                    } else {
                        lobby.join_code_input.clone()
                    };
                    let text_color = if lobby.join_code_input.is_empty() {
                        TEXT_MUTED
                    } else {
                        TEXT_PRIMARY
                    };
                    input.spawn((
                        Text::new(display_text),
                        TextFont::from_font_size(CODE_FONT_SIZE),
                        TextColor(text_color),
                        TextLayout::linebreak(LineBreak::AnyCharacter),
                        Node {
                            max_width: Val::Percent(100.0),
                            ..default()
                        },
                        JoinCodeInputDisplay,
                    ));
                });
            });

        spawn_button(right, "Connect", MpTabAction::ConfirmJoin, &BUTTON_STYLE);
        spawn_button(right, "Cancel", MpTabAction::Cancel, &SMALL_BUTTON_STYLE);
    });
}
