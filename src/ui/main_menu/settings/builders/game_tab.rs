//! Game settings tab content spawn.

use bevy::prelude::*;

use crate::config::GameConfig;

use super::super::components::{ButtonColors, OptionButtonValue, SettingsButtonAction};
use super::super::constants::{
    BUTTON_BACKGROUND, BUTTON_BORDER, BUTTON_BORDER_WIDTH, BUTTON_FONT_SIZE,
    DANGER_BUTTON_BACKGROUND, DANGER_BUTTON_BORDER, LABEL_FONT_SIZE, MARGIN_SMALL,
    OPTION_BUTTON_HEIGHT, OPTION_BUTTON_WIDTH, TEXT_COLOR,
};
use super::setup::{spawn_dot_leader, spawn_option_button, spawn_option_row};

/// Spawns Game tab content: toggles + reset/clear buttons.
pub(super) fn spawn_game_tab(parent: &mut ChildSpawnerCommands, game_config: &GameConfig) {
    let mut wrapper = parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(MARGIN_SMALL),
        ..default()
    });
    wrapper.with_children(|section| {
        spawn_option_row(section, "Skip Splash:", |buttons| {
            spawn_option_button(
                buttons,
                "On",
                OptionButtonValue::SkipSplash(true),
                game_config.skip_splash,
            );
            spawn_option_button(
                buttons,
                "Off",
                OptionButtonValue::SkipSplash(false),
                !game_config.skip_splash,
            );
        });

        spawn_option_row(section, "Tutorials:", |buttons| {
            spawn_option_button(
                buttons,
                "On",
                OptionButtonValue::TutorialsEnabled(true),
                game_config.tutorials_enabled,
            );
            spawn_option_button(
                buttons,
                "Off",
                OptionButtonValue::TutorialsEnabled(false),
                !game_config.tutorials_enabled,
            );
        });

        spawn_option_row(section, "Level Clock:", |buttons| {
            spawn_option_button(
                buttons,
                "On",
                OptionButtonValue::ShowLevelClock(true),
                game_config.show_level_clock,
            );
            spawn_option_button(
                buttons,
                "Off",
                OptionButtonValue::ShowLevelClock(false),
                !game_config.show_level_clock,
            );
        });

        spawn_option_row(section, "Pause on Alt-Tab:", |buttons| {
            spawn_option_button(
                buttons,
                "On",
                OptionButtonValue::AutoPauseOnFocusLoss(true),
                game_config.auto_pause_on_focus_loss,
            );
            spawn_option_button(
                buttons,
                "Off",
                OptionButtonValue::AutoPauseOnFocusLoss(false),
                !game_config.auto_pause_on_focus_loss,
            );
        });

        spawn_option_row(section, "Pause on Steam Overlay:", |buttons| {
            spawn_option_button(
                buttons,
                "On",
                OptionButtonValue::PauseOnSteamOverlay(true),
                game_config.pause_on_steam_overlay,
            );
            spawn_option_button(
                buttons,
                "Off",
                OptionButtonValue::PauseOnSteamOverlay(false),
                !game_config.pause_on_steam_overlay,
            );
        });

        spawn_option_row(section, "Pause on Disconnect:", |buttons| {
            spawn_option_button(
                buttons,
                "On",
                OptionButtonValue::PauseOnControllerDisconnect(true),
                game_config.pause_on_controller_disconnect,
            );
            spawn_option_button(
                buttons,
                "Off",
                OptionButtonValue::PauseOnControllerDisconnect(false),
                !game_config.pause_on_controller_disconnect,
            );
        });

        // Reset Tutorials button
        section
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexEnd,
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("Reset Tutorials:"),
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
                    SettingsButtonAction::ResetTutorials,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Reset"),
                        TextFont::from_font_size(BUTTON_FONT_SIZE),
                        TextColor(TEXT_COLOR),
                    ));
                });
            });

        // Clear Progress button
        section
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexEnd,
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("Clear Progress:"),
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
                    BorderColor::all(DANGER_BUTTON_BORDER),
                    BackgroundColor(DANGER_BUTTON_BACKGROUND),
                    ButtonColors {
                        background: DANGER_BUTTON_BACKGROUND,
                        border: DANGER_BUTTON_BORDER,
                    },
                    crate::ui::focus::Focusable,
                    SettingsButtonAction::ClearProgress,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Clear"),
                        TextFont::from_font_size(BUTTON_FONT_SIZE),
                        TextColor(TEXT_COLOR),
                    ));
                });
            });

        // Dev-only orange "Unlock Everything" button, hidden until F2.
        #[cfg(debug_assertions)]
        super::super::debug_unlock::spawn_unlock_everything_button(section);
    });
}
