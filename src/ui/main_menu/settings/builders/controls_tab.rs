//! Controls tab content spawn: key binding rows, locked subsections, reset button.

use bevy::prelude::*;

use super::super::components::{
    ButtonColors, KeyBindingButton, KeyBindingText, SettingsButtonAction,
};
use super::super::constants::{
    BUTTON_BACKGROUND, BUTTON_BORDER, BUTTON_BORDER_WIDTH, BUTTON_FONT_SIZE, LABEL_FONT_SIZE,
    LOCKED_TEXT_COLOR, LOCKED_TITLE_COLOR, MARGIN, MARGIN_SMALL, OPTION_BUTTON_HEIGHT,
    OPTION_BUTTON_WIDTH, SELECTED_BORDER, TEXT_COLOR,
};
use super::setup::spawn_dot_leader;
use crate::config::input_bindings::{BindingAction, BindingContext, key_display_name};

/// Spawns Controls tab content: key binding subsections + Reset Controls button.
/// Locked wizard archetypes show joke text instead of keybindings.
pub(super) fn spawn_controls_tab(
    parent: &mut ChildSpawnerCommands,
    bindings: &crate::config::InputBindings,
) {
    let unlocked = crate::config::save_data::load_unified_save()
        .map(|s| s.player.unlocked_content.wizard_types)
        .unwrap_or_default();

    let is_unlocked = |wizard_debug_name: &str| -> bool {
        wizard_debug_name == "BoringOleMage" || unlocked.contains(&wizard_debug_name.to_string())
    };

    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(MARGIN_SMALL),
            ..default()
        })
        .with_children(|section| {
            // Universal bindings — always shown
            spawn_controls_subsection(
                section,
                "Universal",
                bindings,
                &[
                    (
                        "Slot 1:",
                        BindingContext::Universal,
                        BindingAction::ActionSlot1,
                    ),
                    (
                        "Slot 2:",
                        BindingContext::Universal,
                        BindingAction::ActionSlot2,
                    ),
                    (
                        "Slot 3:",
                        BindingContext::Universal,
                        BindingAction::ActionSlot3,
                    ),
                    (
                        "Slot 4:",
                        BindingContext::Universal,
                        BindingAction::ActionSlot4,
                    ),
                    (
                        "Slot 5:",
                        BindingContext::Universal,
                        BindingAction::ActionSlot5,
                    ),
                    (
                        "Activate:",
                        BindingContext::Universal,
                        BindingAction::Activate,
                    ),
                ],
            );

            // Archetype-specific bindings — hidden if locked
            if is_unlocked("RuneCaster") {
                spawn_controls_subsection(
                    section,
                    "Rune Caster",
                    bindings,
                    &[
                        ("Rune 1:", BindingContext::RuneCaster, BindingAction::Rune1),
                        ("Rune 2:", BindingContext::RuneCaster, BindingAction::Rune2),
                        ("Rune 3:", BindingContext::RuneCaster, BindingAction::Rune3),
                        ("Rune 4:", BindingContext::RuneCaster, BindingAction::Rune4),
                    ],
                );
            } else {
                spawn_locked_subsection(
                    section,
                    "???",
                    "Try pressing some keys on your keyboard...",
                );
            }

            if is_unlocked("Swordcerer") {
                spawn_controls_subsection(
                    section,
                    "Swordcerer",
                    bindings,
                    &[
                        (
                            "Forward:",
                            BindingContext::Swordcerer,
                            BindingAction::MoveForward,
                        ),
                        (
                            "Backward:",
                            BindingContext::Swordcerer,
                            BindingAction::MoveBackward,
                        ),
                        ("Left:", BindingContext::Swordcerer, BindingAction::MoveLeft),
                        (
                            "Right:",
                            BindingContext::Swordcerer,
                            BindingAction::MoveRight,
                        ),
                    ],
                );
            } else {
                spawn_locked_subsection(
                    section,
                    "???",
                    "Get up close and personal to unlock this one.",
                );
            }

            if is_unlocked("ArcanoRouter") {
                spawn_controls_subsection(
                    section,
                    "Arcanorouter",
                    bindings,
                    &[
                        (
                            "Range +:",
                            BindingContext::ArcanoRouter,
                            BindingAction::RangeUp,
                        ),
                        (
                            "Mana +:",
                            BindingContext::ArcanoRouter,
                            BindingAction::ManaUp,
                        ),
                        (
                            "Power +:",
                            BindingContext::ArcanoRouter,
                            BindingAction::PowerUp,
                        ),
                        (
                            "Speed +:",
                            BindingContext::ArcanoRouter,
                            BindingAction::SpeedUp,
                        ),
                    ],
                );
            } else {
                spawn_locked_subsection(
                    section,
                    "???",
                    "This wizard has a lot of sliders to slide. Unlock to find out.",
                );
            }

            if is_unlocked("Meteorologist") {
                spawn_controls_subsection(
                    section,
                    "Meteorologist",
                    bindings,
                    &[
                        (
                            "Weather 1:",
                            BindingContext::Meteorologist,
                            BindingAction::Weather1,
                        ),
                        (
                            "Weather 2:",
                            BindingContext::Meteorologist,
                            BindingAction::Weather2,
                        ),
                        (
                            "Weather 3:",
                            BindingContext::Meteorologist,
                            BindingAction::Weather3,
                        ),
                    ],
                );
            } else {
                spawn_locked_subsection(
                    section,
                    "???",
                    "Forecast says: locked with a chance of unlocking.",
                );
            }

            if is_unlocked("Warglock") {
                spawn_controls_subsection(
                    section,
                    "Warglock",
                    bindings,
                    &[("Reload:", BindingContext::Warglock, BindingAction::Reload)],
                );
            } else {
                spawn_locked_subsection(section, "???", "Spells are overrated. Find out why.");
            }

            // Reset Controls button
            section
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexEnd,
                    margin: UiRect::top(Val::Px(MARGIN)),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("Reset Controls:"),
                        TextFont::from_font_size(LABEL_FONT_SIZE),
                        TextColor(TEXT_COLOR),
                        Node {
                            flex_shrink: 0.0,
                            ..default()
                        },
                    ));
                    spawn_dot_leader(row);
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(OPTION_BUTTON_WIDTH),
                            height: Val::Px(OPTION_BUTTON_HEIGHT),
                            border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BorderColor::all(BUTTON_BORDER),
                        BackgroundColor(BUTTON_BACKGROUND),
                        ButtonColors {
                            background: BUTTON_BACKGROUND,
                            border: BUTTON_BORDER,
                        },
                        crate::ui::focus::Focusable,
                        SettingsButtonAction::ResetControls,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("Reset"),
                            TextFont::from_font_size(BUTTON_FONT_SIZE),
                            TextColor(TEXT_COLOR),
                        ));
                    });
                });
        });

    // NOTE: The rest of this function was the old placeholder. The spawn_controls_subsection
    // and spawn_key_binding_row helpers are defined below.
}

fn spawn_controls_subsection(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    bindings: &crate::config::InputBindings,
    entries: &[(&str, BindingContext, BindingAction)],
) {
    parent.spawn((
        Text::new(title),
        TextFont::from_font_size(LABEL_FONT_SIZE),
        TextColor(SELECTED_BORDER),
        Node {
            margin: UiRect::top(Val::Px(MARGIN_SMALL)),
            ..default()
        },
    ));
    for &(label, context, action) in entries {
        spawn_key_binding_row(
            parent,
            label,
            context,
            action,
            bindings.get(context, action),
        );
    }
}

fn spawn_locked_subsection(parent: &mut ChildSpawnerCommands, title: &str, joke: &str) {
    parent.spawn((
        Text::new(title),
        TextFont::from_font_size(LABEL_FONT_SIZE),
        TextColor(LOCKED_TITLE_COLOR),
        Node {
            margin: UiRect::top(Val::Px(MARGIN_SMALL)),
            ..default()
        },
    ));
    parent.spawn((
        Text::new(joke),
        TextFont::from_font_size(LABEL_FONT_SIZE),
        TextColor(LOCKED_TEXT_COLOR),
        Node {
            margin: UiRect::left(Val::Px(20.0)),
            ..default()
        },
    ));
}

pub(super) fn spawn_key_binding_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    context: BindingContext,
    action: BindingAction,
    current_key: Option<KeyCode>,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexEnd,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont::from_font_size(LABEL_FONT_SIZE),
                TextColor(TEXT_COLOR),
                Node {
                    flex_shrink: 0.0,
                    ..default()
                },
            ));
            spawn_dot_leader(row);
            row.spawn((
                Button,
                Node {
                    width: Val::Px(OPTION_BUTTON_WIDTH),
                    height: Val::Px(OPTION_BUTTON_HEIGHT),
                    border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BorderColor::all(BUTTON_BORDER),
                BackgroundColor(BUTTON_BACKGROUND),
                ButtonColors {
                    background: BUTTON_BACKGROUND,
                    border: BUTTON_BORDER,
                },
                crate::ui::focus::Focusable,
                KeyBindingButton { context, action },
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new(key_display_name(current_key)),
                    TextFont::from_font_size(BUTTON_FONT_SIZE),
                    TextColor(TEXT_COLOR),
                    KeyBindingText { context, action },
                ));
            });
        });
}
