use bevy::prelude::*;

use crate::config::save_data;
use crate::game::game_mode::components::ToggleModifier;
use crate::ui::components::{ButtonActive, ButtonColors};
use crate::ui::constants::{TEXT_DISABLED, TEXT_MUTED};
use crate::ui::systems::spawn_button;

use super::components::{
    ConfirmUnlockAction, ConfirmUnlockPopup, ToggleDescriptionNode, ToggleExpandButton,
    ToggleRowContainer, ToggleUnlockButton,
};
use super::constants::{
    CANCEL_BUTTON_STYLE, CONFIRM_BUTTON_STYLE, DESCRIPTION_COLOR, LABEL_COLOR, POPUP_BOX_BG,
    POPUP_BOX_BORDER, POPUP_FONT_SIZE, POPUP_OVERLAY_BG, TOGGLE_DESC_FONT_SIZE, TOGGLE_LOCKED_BG,
    TOGGLE_LOCKED_BORDER, TOGGLE_NAME_FONT_SIZE, TOGGLE_OFF_BG, TOGGLE_OFF_BORDER, TOGGLE_ON_BG,
    TOGGLE_ON_BORDER, TOGGLE_SMALL_BUTTON_FONT_SIZE,
};

/// Spawns a single toggle modifier row.
pub(super) fn spawn_toggle_row(
    parent: &mut ChildSpawnerCommands,
    toggle: ToggleModifier,
    is_unlocked: bool,
    is_enabled: bool,
) {
    let (bg, border) = if !is_unlocked {
        (TOGGLE_LOCKED_BG, TOGGLE_LOCKED_BORDER)
    } else if is_enabled {
        (TOGGLE_ON_BG, TOGGLE_ON_BORDER)
    } else {
        (TOGGLE_OFF_BG, TOGGLE_OFF_BORDER)
    };

    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            column_gap: Val::Px(8.0),
            margin: UiRect::bottom(Val::Px(4.0)),
            ..default()
        })
        .with_children(|row| {
            // Toggle button (takes remaining space)
            row.spawn((
                Button,
                Node {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(6.0)),
                    row_gap: Val::Px(4.0),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(bg),
                BorderColor::all(border),
                ButtonColors {
                    background: bg,
                    border,
                },
                ToggleRowContainer(toggle),
                crate::ui::focus::Focusable,
            ))
            .insert_if(ButtonActive, || is_enabled)
            .with_children(|toggle_btn| {
                // Header: [Name ... (Insight cost if locked)]
                toggle_btn
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|header| {
                        let name_color = if is_unlocked {
                            LABEL_COLOR
                        } else {
                            TEXT_DISABLED
                        };
                        header.spawn((
                            Text::new(toggle.display_name()),
                            TextFont::from_font_size(TOGGLE_NAME_FONT_SIZE),
                            TextColor(name_color),
                            Node {
                                flex_grow: 1.0,
                                ..default()
                            },
                        ));

                        if !is_unlocked {
                            header.spawn((
                                Text::new(format!("{} Insight", toggle.insight_cost())),
                                TextFont::from_font_size(TOGGLE_SMALL_BUTTON_FONT_SIZE),
                                TextColor(crate::ui::constants::INSIGHT_COLOR),
                                ToggleUnlockButton(toggle),
                            ));
                        }
                    });

                // Description (always present, revealed when expanded)
                toggle_btn.spawn((
                    Text::new(toggle.description()),
                    TextFont::from_font_size(TOGGLE_DESC_FONT_SIZE),
                    TextColor(DESCRIPTION_COLOR),
                    Node {
                        display: Display::None,
                        max_width: Val::Percent(95.0),
                        padding: UiRect::left(Val::Px(4.0)),
                        ..default()
                    },
                    ToggleDescriptionNode(toggle),
                ));
            });

            // Expand caret button (▾) — matches toggle row height
            let expand_bg = Color::hsla(270.0, 0.10, 0.10, 0.6);
            let expand_border = Color::hsla(270.0, 0.10, 0.20, 0.5);
            row.spawn((
                Button,
                Node {
                    width: Val::Px(36.0),
                    min_height: Val::Px(36.0),
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BorderColor::all(expand_border),
                BackgroundColor(expand_bg),
                ButtonColors {
                    background: expand_bg,
                    border: expand_border,
                },
                ToggleExpandButton(toggle),
                crate::ui::focus::Focusable,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("?"),
                    TextFont::from_font_size(14.0),
                    TextColor(TEXT_MUTED),
                ));
            });
        });
}

/// Spawns a confirmation popup for unlocking a toggle modifier.
pub(super) fn spawn_unlock_popup(commands: &mut Commands, toggle: ToggleModifier) {
    let current_insight = save_data::get_insight();
    let cost = toggle.insight_cost();
    let can_afford = current_insight >= cost;

    let message = if can_afford {
        format!(
            "Unlock \"{}\" for {} Insight?\n(You have {} Insight)",
            toggle.display_name(),
            cost,
            current_insight
        )
    } else {
        format!(
            "Not enough Insight to unlock \"{}\".\nCost: {} | You have: {}",
            toggle.display_name(),
            cost,
            current_insight
        )
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(POPUP_OVERLAY_BG),
            GlobalZIndex(600),
            ConfirmUnlockPopup,
            crate::ui::focus::ModalOverlay,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(30.0)),
                        row_gap: Val::Px(20.0),
                        border: UiRect::all(Val::Px(2.0)),
                        min_width: Val::Px(350.0),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(POPUP_BOX_BG),
                    BorderColor::all(POPUP_BOX_BORDER),
                ))
                .with_children(|popup| {
                    popup.spawn((
                        Text::new(message),
                        TextFont::from_font_size(POPUP_FONT_SIZE),
                        TextColor(LABEL_COLOR),
                        TextLayout::new_with_justify(Justify::Center),
                    ));

                    popup
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(16.0),
                            ..default()
                        })
                        .with_children(|buttons| {
                            if can_afford {
                                spawn_button(
                                    buttons,
                                    "Confirm",
                                    ConfirmUnlockAction::Confirm(toggle),
                                    &CONFIRM_BUTTON_STYLE,
                                );
                            }
                            spawn_button(
                                buttons,
                                "Cancel",
                                ConfirmUnlockAction::Cancel,
                                &CANCEL_BUTTON_STYLE,
                            );
                        });
                });
        });
}
