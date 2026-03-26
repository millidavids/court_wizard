//! Settings menu systems.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::config::{DisplayMode, GameConfig, VsyncMode};
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::state::{MenuState, PauseMenuState};
use crate::ui::styles::{item_hovered, item_pressed};
use crate::ui::systems::spawn_title_with_shadow;

use super::components::{
    ButtonColors, ConfirmationAction, ConfirmationPopup, OnSettingsScreen, OptionButtonValue,
    ScrollableContainer, SelectedOption, SettingsButtonAction, SliderAdjusted, SliderDownButton,
    SliderFill, SliderHandle, SliderText, SliderTrack, SliderUpButton, SliderValue,
};
use super::constants::{
    BUTTON_BACKGROUND, BUTTON_BORDER, BUTTON_BORDER_WIDTH,
    BUTTON_FONT_SIZE, DANGER_BUTTON_BACKGROUND, DANGER_BUTTON_BORDER, LABEL_FONT_SIZE, MARGIN,
    MARGIN_SMALL, OPTION_BUTTON_HEIGHT, OPTION_BUTTON_WIDTH, POPUP_BOX_BG, POPUP_OVERLAY_BG,
    SECTION_FONT_SIZE, SELECTED_BACKGROUND, SELECTED_BORDER, TEXT_COLOR, TITLE_FONT_SIZE,
    VOLUME_BUTTON_SIZE,
};

/// Sets up the settings menu UI.
///
/// Creates a scrollable settings screen with controls for:
/// - VSync mode (On, Off, Adaptive)
/// - Audio volumes (Master, Music, SFX)
/// - Game difficulty (Easy, Normal, Hard)
///
/// All spawned entities are marked with `OnSettingsScreen` for cleanup.
///
/// # Arguments
///
/// * `commands` - Bevy command buffer for spawning entities
/// * `game_config` - Current game configuration
fn setup(mut commands: Commands, game_config: Res<GameConfig>, pause_menu: bool) {
    use crate::ui::systems::spawn_page_container;

    let content = spawn_page_container(
        &mut commands,
        OnSettingsScreen,
        pause_menu,
        Overflow::clip(),
    );
    commands.entity(content).with_children(|parent| {
        // Title
        spawn_title_with_shadow(parent, "Settings", TITLE_FONT_SIZE, TEXT_COLOR, Node {
            margin: UiRect::bottom(Val::Px(MARGIN)),
            ..default()
        });

        // Scrollable settings content
        parent
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    overflow: Overflow::scroll_y(),
                    margin: UiRect::bottom(Val::Px(MARGIN)),
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                crate::ui::systems::scroll_area_style(),
                ScrollPosition::default(),
                ScrollableContainer,
            ))
            .with_children(|scroll| {
            // Graphics Settings Section
            spawn_section(scroll, "Graphics", |section| {
                // VSync Mode
                spawn_option_row(section, "VSync:", |buttons| {
                    spawn_option_button(
                        buttons,
                        "On",
                        OptionButtonValue::VsyncMode(VsyncMode::On),
                        game_config.vsync == VsyncMode::On,
                    );
                    spawn_option_button(
                        buttons,
                        "Off",
                        OptionButtonValue::VsyncMode(VsyncMode::Off),
                        game_config.vsync == VsyncMode::Off,
                    );
                    spawn_option_button(
                        buttons,
                        "Adaptive",
                        OptionButtonValue::VsyncMode(VsyncMode::Adaptive),
                        game_config.vsync == VsyncMode::Adaptive,
                    );
                });

                // Display Mode
                spawn_option_row(section, "Display Mode:", |buttons| {
                    spawn_option_button(
                        buttons,
                        "Windowed",
                        OptionButtonValue::DisplayMode(DisplayMode::Windowed),
                        game_config.display_mode == DisplayMode::Windowed,
                    );
                    spawn_option_button(
                        buttons,
                        "Fullscreen",
                        OptionButtonValue::DisplayMode(DisplayMode::BorderlessFullscreen),
                        game_config.display_mode == DisplayMode::BorderlessFullscreen,
                    );
                });
            });

            // Audio Settings Section
            spawn_section(scroll, "Audio", |section| {
                spawn_slider_control(
                    section,
                    "Master Volume:",
                    SliderValue::MasterVolume,
                    &game_config,
                );
                spawn_slider_control(
                    section,
                    "Music Volume:",
                    SliderValue::MusicVolume,
                    &game_config,
                );
                spawn_slider_control(section, "SFX Volume:", SliderValue::SfxVolume, &game_config);
            });

            // Display Settings Section
            spawn_section(scroll, "Display", |section| {
                spawn_slider_control(
                    section,
                    "Brightness:",
                    SliderValue::UiBrightness,
                    &game_config,
                );
            });

            // Game Settings Section
            spawn_section(scroll, "Game", |section| {
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

                spawn_option_row(section, "Urgent Mode:", |buttons| {
                    spawn_option_button(
                        buttons,
                        "On",
                        OptionButtonValue::UrgentMode(true),
                        game_config.urgent_mode,
                    );
                    spawn_option_button(
                        buttons,
                        "Off",
                        OptionButtonValue::UrgentMode(false),
                        !game_config.urgent_mode,
                    );
                });

                // Reset Tutorials button
                section
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(MARGIN),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new("Reset Tutorials:"),
                            TextFont::from_font_size(LABEL_FONT_SIZE),
                            TextColor(TEXT_COLOR),
                            Node {
                                width: Val::Px(200.0),
                                ..default()
                            },
                        ));

                        row.spawn((
                            Button,
                            Node {
                                width: Val::Px(OPTION_BUTTON_WIDTH),
                                height: Val::Px(OPTION_BUTTON_HEIGHT),
                                border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor::all(BUTTON_BORDER),
                            BorderRadius::all(Val::Px(4.0)),
                            BackgroundColor(BUTTON_BACKGROUND),
                            ButtonColors {
                                background: BUTTON_BACKGROUND,
                            },
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
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(MARGIN),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new("Clear Progress:"),
                            TextFont::from_font_size(LABEL_FONT_SIZE),
                            TextColor(TEXT_COLOR),
                            Node {
                                width: Val::Px(200.0),
                                ..default()
                            },
                        ));

                        row.spawn((
                            Button,
                            Node {
                                width: Val::Px(OPTION_BUTTON_WIDTH),
                                height: Val::Px(OPTION_BUTTON_HEIGHT),
                                border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor::all(DANGER_BUTTON_BORDER),
                            BorderRadius::all(Val::Px(4.0)),
                            BackgroundColor(DANGER_BUTTON_BACKGROUND),
                            ButtonColors {
                                background: DANGER_BUTTON_BACKGROUND,
                            },
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
            });
            }); // end scrollable container

        // Back button (outside scroll area)
        parent
            .spawn((
                Button,
                Node {
                    width: Val::Px(150.0),
                    height: Val::Px(50.0),
                    border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(BUTTON_BORDER),
                BorderRadius::all(Val::Px(8.0)),
                BackgroundColor(BUTTON_BACKGROUND),
                ButtonColors {
                    background: BUTTON_BACKGROUND,
                },
                SettingsButtonAction::Back,
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new("Back"),
                    TextFont::from_font_size(18.0),
                    TextColor(TEXT_COLOR),
                ));
            });
    });
}

/// Spawns settings with opaque background (for main menu).
pub fn setup_main_menu(commands: Commands, game_config: Res<GameConfig>) {
    setup(commands, game_config, false);
}

/// Spawns settings with transparent background and GlobalZIndex (for pause menu).
pub fn setup_pause_menu(commands: Commands, game_config: Res<GameConfig>) {
    setup(commands, game_config, true);
}

/// Helper function to spawn a settings section with a title.
fn spawn_section(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    spawn_content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(MARGIN_SMALL),
            margin: UiRect::vertical(Val::Px(MARGIN)),
            ..default()
        })
        .with_children(|section| {
            // Section title
            section.spawn((
                Text::new(title),
                TextFont::from_font_size(SECTION_FONT_SIZE),
                TextColor(TEXT_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(MARGIN_SMALL)),
                    ..default()
                },
            ));

            spawn_content(section);
        });
}

/// Helper function to spawn a row with a label and option buttons.
fn spawn_option_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    spawn_buttons: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(MARGIN),
            ..default()
        })
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new(label),
                TextFont::from_font_size(LABEL_FONT_SIZE),
                TextColor(TEXT_COLOR),
                Node {
                    width: Val::Px(200.0),
                    ..default()
                },
            ));

            // Buttons container
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(MARGIN_SMALL),
                ..default()
            })
            .with_children(spawn_buttons);
        });
}

/// Helper function to spawn an option button.
fn spawn_option_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    value: OptionButtonValue,
    is_selected: bool,
) {
    let (bg_color, border_color) = if is_selected {
        (SELECTED_BACKGROUND, SELECTED_BORDER)
    } else {
        (BUTTON_BACKGROUND, BUTTON_BORDER)
    };

    let mut entity = parent.spawn((
        Button,
        Node {
            width: Val::Px(OPTION_BUTTON_WIDTH),
            height: Val::Px(OPTION_BUTTON_HEIGHT),
            border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor::all(border_color),
        BorderRadius::all(Val::Px(4.0)),
        BackgroundColor(bg_color),
        ButtonColors {
            background: bg_color,
        },
        value,
    ));

    if is_selected {
        entity.insert(SelectedOption);
    }

    entity.with_children(|button| {
        // Shrink font for longer labels that don't fit the button width
        let font_size = crate::ui::systems::scale_font_by_text_width(
            text.len() as f32,
            6.0,  // up to 6 chars fits comfortably at full size
            12.0, // 12+ chars gets minimum scale
            0.7,  // minimum 70% of base font
            BUTTON_FONT_SIZE,
        );
        button.spawn((
            Text::new(text),
            TextFont::from_font_size(font_size),
            TextColor(TEXT_COLOR),
        ));
    });
}

/// Configuration for spawning a slider row.
struct SliderRowConfig<'a, TText, TDownButton, TUpButton, TSliderTrack, TSliderFill, TSliderHandle>
{
    label: &'a str,
    current_value: f32,
    max_value: f32,
    text_component: TText,
    down_button: TDownButton,
    up_button: TUpButton,
    slider_track: TSliderTrack,
    slider_fill: TSliderFill,
    slider_handle: TSliderHandle,
}

/// Helper function to spawn a slider row with decrease/increase buttons, slider, and value display.
fn spawn_slider_row<
    TText: Component,
    TDownButton: Component,
    TUpButton: Component,
    TSliderTrack: Component,
    TSliderFill: Component,
    TSliderHandle: Component,
>(
    parent: &mut ChildSpawnerCommands,
    config: SliderRowConfig<
        '_,
        TText,
        TDownButton,
        TUpButton,
        TSliderTrack,
        TSliderFill,
        TSliderHandle,
    >,
) {
    let SliderRowConfig {
        label,
        current_value,
        max_value,
        text_component,
        down_button,
        up_button,
        slider_track,
        slider_fill,
        slider_handle,
    } = config;
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(MARGIN),
            ..default()
        })
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new(label),
                TextFont::from_font_size(LABEL_FONT_SIZE),
                TextColor(TEXT_COLOR),
                Node {
                    width: Val::Px(200.0),
                    ..default()
                },
            ));

            // Controls
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(MARGIN_SMALL),
                ..default()
            })
            .with_children(|controls| {
                // Decrease button
                controls
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(VOLUME_BUTTON_SIZE),
                            height: Val::Px(VOLUME_BUTTON_SIZE),
                            border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor::all(BUTTON_BORDER),
                        BorderRadius::all(Val::Px(4.0)),
                        BackgroundColor(BUTTON_BACKGROUND),
                        ButtonColors {
                            background: BUTTON_BACKGROUND,
                        },
                        down_button,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("-"),
                            TextFont::from_font_size(BUTTON_FONT_SIZE),
                            TextColor(TEXT_COLOR),
                        ));
                    });

                // Slider track
                controls
                    .spawn((
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(12.0),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            position_type: PositionType::Relative,
                            ..default()
                        },
                        BorderColor::all(BUTTON_BORDER),
                        BorderRadius::all(Val::Px(6.0)),
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                        Interaction::default(),
                        RelativeCursorPosition::default(),
                        slider_track,
                    ))
                    .with_children(|track| {
                        // Slider fill
                        let normalized = current_value / max_value;
                        track.spawn((
                            Node {
                                width: Val::Percent(normalized * 100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BorderRadius {
                                top_left: Val::Px(6.0),
                                bottom_left: Val::Px(6.0),
                                top_right: Val::Px(0.0),
                                bottom_right: Val::Px(0.0),
                            },
                            BackgroundColor(BUTTON_BORDER),
                            slider_fill,
                        ));

                        // Slider handle (offset by -2px to center the 4px wide bar)
                        track.spawn((
                            Node {
                                width: Val::Px(4.0),
                                height: Val::Px(20.0),
                                position_type: PositionType::Absolute,
                                left: Val::Px(normalized * 200.0 - 2.0),
                                top: Val::Px(-4.0),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(2.0)),
                            BackgroundColor(Color::WHITE),
                            BorderColor::all(BUTTON_BORDER),
                            Interaction::default(),
                            RelativeCursorPosition::default(),
                            slider_handle,
                        ));
                    });

                // Increase button
                controls
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(VOLUME_BUTTON_SIZE),
                            height: Val::Px(VOLUME_BUTTON_SIZE),
                            border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor::all(BUTTON_BORDER),
                        BorderRadius::all(Val::Px(4.0)),
                        BackgroundColor(BUTTON_BACKGROUND),
                        ButtonColors {
                            background: BUTTON_BACKGROUND,
                        },
                        up_button,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("+"),
                            TextFont::from_font_size(BUTTON_FONT_SIZE),
                            TextColor(TEXT_COLOR),
                        ));
                    });

                // Value display
                controls.spawn((
                    Text::new(format!("{}%", (current_value * 100.0) as u8)),
                    TextFont::from_font_size(LABEL_FONT_SIZE),
                    TextColor(TEXT_COLOR),
                    Node {
                        width: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    text_component,
                ));
            });
        });
}

/// Helper function to spawn a slider control row.
fn spawn_slider_control(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    slider_value: SliderValue,
    game_config: &GameConfig,
) {
    let current_value = slider_value.get(game_config);
    let max_value = slider_value.max_value();

    spawn_slider_row(
        parent,
        SliderRowConfig {
            label,
            current_value,
            max_value,
            text_component: SliderText {
                value: slider_value,
            },
            down_button: SliderDownButton {
                value: slider_value,
            },
            up_button: SliderUpButton {
                value: slider_value,
            },
            slider_track: SliderTrack {
                value: slider_value,
            },
            slider_fill: SliderFill {
                value: slider_value,
            },
            slider_handle: SliderHandle {
                value: slider_value,
                is_dragging: false,
            },
        },
    );
}

/// Handles button hover visual feedback.
///
/// Changes button colors when the cursor hovers over them.
///
/// # Arguments
///
/// * `interactions` - Query for button interaction states
pub fn button_hover(
    mut interactions: Query<
        (&Interaction, &ButtonColors, &mut BackgroundColor),
        (Changed<Interaction>, Without<SelectedOption>),
    >,
) {
    for (interaction, colors, mut background) in &mut interactions {
        match interaction {
            Interaction::Hovered => *background = BackgroundColor(item_hovered(colors.background)),
            Interaction::None => *background = BackgroundColor(colors.background),
            _ => {}
        }
    }
}

/// Handles button press visual feedback.
///
/// Changes button colors when buttons are pressed.
///
/// # Arguments
///
/// * `interactions` - Query for button interaction states
pub fn button_press(
    mut interactions: Query<
        (&Interaction, &ButtonColors, &mut BackgroundColor),
        (Changed<Interaction>, Without<SelectedOption>),
    >,
) {
    for (interaction, colors, mut background) in &mut interactions {
        if *interaction == Interaction::Pressed {
            *background = BackgroundColor(item_pressed(colors.background));
        }
    }
}

/// Spawns a confirmation popup overlay in the center of the screen.
fn spawn_confirmation_popup(commands: &mut Commands, action: SettingsButtonAction, message: &str) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(POPUP_OVERLAY_BG),
            GlobalZIndex(600),
            ConfirmationPopup,
            OnSettingsScreen,
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
                        ..default()
                    },
                    BackgroundColor(POPUP_BOX_BG),
                    BorderColor::all(DANGER_BUTTON_BORDER),
                    BorderRadius::all(Val::Px(8.0)),
                ))
                .with_children(|popup| {
                    popup.spawn((
                        Text::new(message),
                        TextFont::from_font_size(SECTION_FONT_SIZE),
                        TextColor(TEXT_COLOR),
                    ));

                    popup
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(MARGIN),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                Node {
                                    width: Val::Px(OPTION_BUTTON_WIDTH),
                                    height: Val::Px(OPTION_BUTTON_HEIGHT),
                                    border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BorderColor::all(DANGER_BUTTON_BORDER),
                                BorderRadius::all(Val::Px(4.0)),
                                BackgroundColor(DANGER_BUTTON_BACKGROUND),
                                ButtonColors {
                                    background: DANGER_BUTTON_BACKGROUND,
                                },
                                ConfirmationAction::Confirm(action),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Confirm"),
                                    TextFont::from_font_size(BUTTON_FONT_SIZE),
                                    TextColor(TEXT_COLOR),
                                ));
                            });

                            row.spawn((
                                Button,
                                Node {
                                    width: Val::Px(OPTION_BUTTON_WIDTH),
                                    height: Val::Px(OPTION_BUTTON_HEIGHT),
                                    border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BorderColor::all(BUTTON_BORDER),
                                BorderRadius::all(Val::Px(4.0)),
                                BackgroundColor(BUTTON_BACKGROUND),
                                ButtonColors {
                                    background: BUTTON_BACKGROUND,
                                },
                                ConfirmationAction::Cancel,
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Cancel"),
                                    TextFont::from_font_size(BUTTON_FONT_SIZE),
                                    TextColor(TEXT_COLOR),
                                ));
                            });
                        });
                });
        });
}

/// Handles confirm/cancel clicks on the confirmation popup.
pub fn handle_confirmation_popup(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    action_query: Query<&ConfirmationAction>,
    popup_query: Query<Entity, With<ConfirmationPopup>>,
    mut tutorial_progress: ResMut<crate::ui::tutorial::resources::TutorialProgress>,
    mut popup_queue: ResMut<crate::ui::achievement_popup::PopupQueue>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = action_query.get(event.button) {
            if let ConfirmationAction::Confirm(settings_action) = action {
                match settings_action {
                    SettingsButtonAction::ResetTutorials => {
                        tutorial_progress.reset();
                        crate::ui::tutorial::systems::reset_tutorial_progress();
                        popup_queue.push(crate::ui::achievement_popup::PopupEntry::Toast {
                            message: "Tutorials have been reset.",
                        });
                    }
                    SettingsButtonAction::ClearProgress => {
                        crate::config::save_data::clear_progress();
                        popup_queue.push(crate::ui::achievement_popup::PopupEntry::Toast {
                            message: "All progress has been cleared.",
                        });
                    }
                    _ => {}
                }
            }
            // Despawn popup on either Confirm or Cancel
            for entity in &popup_query {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Handles settings button actions when clicked from main menu.
pub fn settings_button_action(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&SettingsButtonAction>,
    popup_query: Query<&ConfirmationPopup>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    // Don't process settings buttons while a confirmation popup is open
    if !popup_query.is_empty() {
        button_clicked.read();
        return;
    }
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                SettingsButtonAction::Back => {
                    channel_change.write(ChannelChangeMessage);
                    next_menu_state.set(MenuState::Landing);
                }
                SettingsButtonAction::ResetTutorials => {
                    spawn_confirmation_popup(
                        &mut commands,
                        *action,
                        "Reset all tutorials?",
                    );
                }
                SettingsButtonAction::ClearProgress => {
                    spawn_confirmation_popup(
                        &mut commands,
                        *action,
                        "Clear all progress? This cannot be undone.",
                    );
                }
            }
        }
    }
}

/// Handles settings button actions when clicked from pause menu.
pub fn pause_settings_button_action(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&SettingsButtonAction>,
    popup_query: Query<&ConfirmationPopup>,
    mut next_pause_menu_state: ResMut<NextState<PauseMenuState>>,
) {
    // Don't process settings buttons while a confirmation popup is open
    if !popup_query.is_empty() {
        button_clicked.read();
        return;
    }
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                SettingsButtonAction::Back => {
                    next_pause_menu_state.set(PauseMenuState::Main);
                }
                SettingsButtonAction::ResetTutorials => {
                    spawn_confirmation_popup(
                        &mut commands,
                        *action,
                        "Reset all tutorials?",
                    );
                }
                SettingsButtonAction::ClearProgress => {
                    spawn_confirmation_popup(
                        &mut commands,
                        *action,
                        "Clear all progress? This cannot be undone.",
                    );
                }
            }
        }
    }
}

/// Handles option button clicks.
pub fn option_button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&OptionButtonValue>,
    mut game_config: ResMut<GameConfig>,
) {
    for event in button_clicked.read() {
        if let Ok(value) = button_query.get(event.button) {
            value.apply(&mut game_config);
        }
    }
}

/// Handles slider button clicks for increment/decrement.
pub fn slider_button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    down_buttons: Query<&SliderDownButton>,
    up_buttons: Query<&SliderUpButton>,
    mut game_config: ResMut<GameConfig>,
    mut slider_adjusted: MessageWriter<SliderAdjusted>,
) {
    for event in button_clicked.read() {
        // Check if it's a down button
        if let Ok(button) = down_buttons.get(event.button) {
            let current = button.value.get(&game_config);
            let step = button.value.step();
            let min = button.value.min_value();
            let new_value = (current - step).max(min);
            button.value.set(&mut game_config, new_value);
            slider_adjusted.write(SliderAdjusted);
        }
        // Check if it's an up button
        else if let Ok(button) = up_buttons.get(event.button) {
            let current = button.value.get(&game_config);
            let step = button.value.step();
            let max = button.value.max_value();
            let new_value = (current + step).min(max);
            button.value.set(&mut game_config, new_value);
            slider_adjusted.write(SliderAdjusted);
        }
    }
}

/// Updates slider text displays when values change.
pub fn update_slider_text(
    game_config: Res<GameConfig>,
    mut slider_texts: Query<(&mut Text, &SliderText)>,
) {
    if game_config.is_changed() {
        for (mut text, slider_text) in &mut slider_texts {
            let value = slider_text.value.get(&game_config);
            text.0 = format!("{}%", (value * 100.0) as u8);
        }
    }
}

/// Updates slider fill widths and handle positions when values change.
pub fn update_sliders(
    game_config: Res<GameConfig>,
    mut slider_fills: Query<(&mut Node, &SliderFill), Without<SliderHandle>>,
    mut slider_handles: Query<(&mut Node, &SliderHandle), Without<SliderFill>>,
) {
    if game_config.is_changed() {
        for (mut node, slider_fill) in &mut slider_fills {
            let value = slider_fill.value.get(&game_config);
            let min = slider_fill.value.min_value();
            let max = slider_fill.value.max_value();
            let range = max - min;
            // Normalize to 0-100% range
            let normalized = (value - min) / range;
            node.width = Val::Percent(normalized * 100.0);
        }

        for (mut node, slider_handle) in &mut slider_handles {
            let value = slider_handle.value.get(&game_config);
            let min = slider_handle.value.min_value();
            let max = slider_handle.value.max_value();
            let range = max - min;
            // Center the handle on the position (200px track width, -2px offset for 4px handle)
            let normalized = (value - min) / range;
            node.left = Val::Px(normalized * 200.0 - 2.0);
        }
    }
}

/// Handles dragging slider handles and clicking on tracks.
///
/// Uses the track's `RelativeCursorPosition` for both click-to-jump and drag
/// tracking. This is immune to scale factor, viewport, and CRT distortion
/// differences between mouse motion units and logical UI pixels.
pub fn slider_interaction(
    buttons: Res<ButtonInput<bevy::input::mouse::MouseButton>>,
    mut slider_handles: Query<(&Interaction, &mut SliderHandle)>,
    slider_tracks: Query<(&Interaction, &RelativeCursorPosition, &SliderTrack)>,
    mut game_config: ResMut<GameConfig>,
    mut slider_adjusted: MessageWriter<SliderAdjusted>,
) {
    // Stop dragging when mouse is released
    if !buttons.pressed(bevy::input::mouse::MouseButton::Left) {
        for (_interaction, mut slider_handle) in &mut slider_handles {
            slider_handle.is_dragging = false;
        }
        return;
    }

    // Check if track was clicked (start dragging)
    if buttons.just_pressed(bevy::input::mouse::MouseButton::Left) {
        for (interaction, _cursor_pos, track) in &slider_tracks {
            if matches!(*interaction, Interaction::Pressed | Interaction::Hovered) {
                // Start dragging the corresponding handle
                for (_handle_interaction, mut slider_handle) in &mut slider_handles {
                    if slider_handle.value == track.value {
                        slider_handle.is_dragging = true;
                    }
                }
            }
        }

        // Also start dragging if the handle itself was clicked
        for (interaction, mut slider_handle) in &mut slider_handles {
            if *interaction == Interaction::Pressed {
                slider_handle.is_dragging = true;
            }
        }
    }

    // While dragging, use the track's RelativeCursorPosition to set the value.
    // This gives pixel-perfect tracking regardless of scale factor or viewport.
    for (_interaction, cursor_pos, track) in &slider_tracks {
        let is_dragging = slider_handles
            .iter()
            .any(|(_, h)| h.value == track.value && h.is_dragging);

        if is_dragging {
            if let Some(pos) = cursor_pos.normalized {
                // RelativeCursorPosition.normalized: center at (0,0),
                // left edge = -0.5, right edge = 0.5
                let normalized = (pos.x + 0.5).clamp(0.0, 1.0);

                let min = track.value.min_value();
                let max = track.value.max_value();
                let range = max - min;
                let new_value = (min + normalized * range).clamp(min, max);

                if (track.value.get(&game_config) - new_value).abs() > f32::EPSILON {
                    track.value.set(&mut game_config, new_value);
                    slider_adjusted.write(SliderAdjusted);
                }
            }
        }
    }
}

/// Updates selected state styling for option buttons.
pub fn update_selected_options(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    mut option_buttons: Query<
        (
            Entity,
            &OptionButtonValue,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
) {
    if game_config.is_changed() {
        for (entity, value, mut bg, mut border) in &mut option_buttons {
            if value.is_selected(&game_config) {
                commands.entity(entity).insert(SelectedOption);
                *bg = BackgroundColor(SELECTED_BACKGROUND);
                *border = BorderColor::all(SELECTED_BORDER);
            } else {
                commands.entity(entity).remove::<SelectedOption>();
                *bg = BackgroundColor(BUTTON_BACKGROUND);
                *border = BorderColor::all(BUTTON_BORDER);
            }
        }
    }
}
