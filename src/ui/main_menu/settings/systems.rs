//! Settings menu systems.

use bevy::ecs::relationship::Relationship;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::config::{Difficulty, GameConfig, VsyncMode};
use crate::state::MenuState;
use crate::ui::styles::{item_hovered, item_pressed};

use super::components::{
    ButtonColors, DifficultyButton, OnSettingsScreen, ScrollableContainer, SelectedOption,
    SettingsButtonAction, UiBrightnessDownButton, UiBrightnessText, UiBrightnessUpButton,
    VolumeDownButton, VolumeSliderFill, VolumeSliderHandle, VolumeSliderTrack, VolumeText,
    VolumeType, VolumeUpButton, VsyncModeButton,
};
use super::styles::{
    BACK_BUTTON_HEIGHT, BACK_BUTTON_WIDTH, BUTTON_BACKGROUND, BUTTON_BORDER, BUTTON_BORDER_WIDTH,
    BUTTON_FONT_SIZE, LABEL_FONT_SIZE, MARGIN, MARGIN_SMALL, OPTION_BUTTON_HEIGHT,
    OPTION_BUTTON_WIDTH, SECTION_FONT_SIZE, SELECTED_BACKGROUND, SELECTED_BORDER, TEXT_COLOR,
    TITLE_FONT_SIZE, VOLUME_BUTTON_SIZE,
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
pub fn setup(mut commands: Commands, game_config: Res<GameConfig>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition::default(),
            OnSettingsScreen,
            ScrollableContainer,
        ))
        .with_children(|parent| {
            // Content container
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(MARGIN * 2.0)),
                    row_gap: Val::Px(MARGIN),
                    ..default()
                })
                .with_children(|parent| {
                    // Title
                    parent.spawn((
                        Text::new("Settings"),
                        TextFont {
                            font_size: TITLE_FONT_SIZE,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        Node {
                            margin: UiRect::bottom(Val::Px(MARGIN)),
                            ..default()
                        },
                    ));

                    // Graphics Settings Section
                    spawn_section(parent, "Graphics", |section| {
                        // VSync Mode
                        spawn_option_row(section, "VSync:", |buttons| {
                            spawn_option_button(
                                buttons,
                                "On",
                                SettingsButtonAction::SetVsyncOn,
                                game_config.vsync == VsyncMode::On,
                                Some(VsyncModeButton(VsyncMode::On)),
                            );
                            spawn_option_button(
                                buttons,
                                "Off",
                                SettingsButtonAction::SetVsyncOff,
                                game_config.vsync == VsyncMode::Off,
                                Some(VsyncModeButton(VsyncMode::Off)),
                            );
                            spawn_option_button(
                                buttons,
                                "Adaptive",
                                SettingsButtonAction::SetVsyncAdaptive,
                                game_config.vsync == VsyncMode::Adaptive,
                                Some(VsyncModeButton(VsyncMode::Adaptive)),
                            );
                        });
                    });

                    // Audio Settings Section
                    spawn_section(parent, "Audio", |section| {
                        spawn_volume_row(
                            section,
                            "Master Volume:",
                            VolumeType::Master,
                            game_config.master_volume,
                        );
                        spawn_volume_row(
                            section,
                            "Music Volume:",
                            VolumeType::Music,
                            game_config.music_volume,
                        );
                        spawn_volume_row(
                            section,
                            "SFX Volume:",
                            VolumeType::Sfx,
                            game_config.sfx_volume,
                        );
                    });

                    // Display Settings Section
                    spawn_section(parent, "Display", |section| {
                        spawn_ui_brightness_row(section, "Brightness:", game_config.brightness);
                    });

                    // Game Settings Section
                    spawn_section(parent, "Game", |section| {
                        spawn_option_row(section, "Difficulty:", |buttons| {
                            spawn_option_button(
                                buttons,
                                "Easy",
                                SettingsButtonAction::SetDifficultyEasy,
                                game_config.difficulty == Difficulty::Easy,
                                Some(DifficultyButton(Difficulty::Easy)),
                            );
                            spawn_option_button(
                                buttons,
                                "Normal",
                                SettingsButtonAction::SetDifficultyNormal,
                                game_config.difficulty == Difficulty::Normal,
                                Some(DifficultyButton(Difficulty::Normal)),
                            );
                            spawn_option_button(
                                buttons,
                                "Hard",
                                SettingsButtonAction::SetDifficultyHard,
                                game_config.difficulty == Difficulty::Hard,
                                Some(DifficultyButton(Difficulty::Hard)),
                            );
                        });
                    });

                    // Back button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(BACK_BUTTON_WIDTH),
                                height: Val::Px(BACK_BUTTON_HEIGHT),
                                border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Px(MARGIN)),
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
                                TextFont {
                                    font_size: BUTTON_FONT_SIZE,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });
                });
        });
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
                TextFont {
                    font_size: SECTION_FONT_SIZE,
                    ..default()
                },
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
                TextFont {
                    font_size: LABEL_FONT_SIZE,
                    ..default()
                },
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
fn spawn_option_button<T: Component>(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    action: SettingsButtonAction,
    is_selected: bool,
    marker: Option<T>,
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
        action,
    ));

    if is_selected {
        entity.insert(SelectedOption);
    }

    if let Some(marker_component) = marker {
        entity.insert(marker_component);
    }

    entity.with_children(|button| {
        button.spawn((
            Text::new(text),
            TextFont {
                font_size: BUTTON_FONT_SIZE,
                ..default()
            },
            TextColor(TEXT_COLOR),
        ));
    });
}

/// Helper function to spawn a volume control row.
fn spawn_volume_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    volume_type: VolumeType,
    current_value: f32,
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
                TextFont {
                    font_size: LABEL_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    width: Val::Px(200.0),
                    ..default()
                },
            ));

            // Volume controls
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
                        VolumeDownButton { volume_type },
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("-"),
                            TextFont {
                                font_size: BUTTON_FONT_SIZE,
                                ..default()
                            },
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
                        RelativeCursorPosition::default(),
                        VolumeSliderTrack { volume_type },
                    ))
                    .with_children(|track| {
                        // Slider fill
                        track.spawn((
                            Node {
                                width: Val::Percent(current_value * 100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(6.0)),
                            BackgroundColor(BUTTON_BORDER),
                            VolumeSliderFill { volume_type },
                        ));

                        // Slider handle (offset by -2px to center the 4px wide bar)
                        track.spawn((
                            Node {
                                width: Val::Px(4.0),
                                height: Val::Px(20.0),
                                position_type: PositionType::Absolute,
                                left: Val::Px(current_value * 200.0 - 2.0), // 200px track width, -2px to center
                                top: Val::Px(-4.0),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(2.0)),
                            BackgroundColor(Color::WHITE),
                            BorderColor::all(BUTTON_BORDER),
                            Interaction::default(),
                            RelativeCursorPosition::default(),
                            VolumeSliderHandle {
                                volume_type,
                                is_dragging: false,
                            },
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
                        VolumeUpButton { volume_type },
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("+"),
                            TextFont {
                                font_size: BUTTON_FONT_SIZE,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        ));
                    });

                // Value display
                controls.spawn((
                    Text::new(format!("{}%", (current_value * 100.0) as u8)),
                    TextFont {
                        font_size: LABEL_FONT_SIZE,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    Node {
                        width: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    VolumeText { volume_type },
                ));
            });
        });
}

/// Spawns a UI brightness control row with decrease/increase buttons and value display.
fn spawn_ui_brightness_row(parent: &mut ChildSpawnerCommands, label: &str, current_value: f32) {
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
                TextFont {
                    font_size: LABEL_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Node {
                    width: Val::Px(200.0),
                    ..default()
                },
            ));

            // UI brightness controls
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
                        UiBrightnessDownButton,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("-"),
                            TextFont {
                                font_size: BUTTON_FONT_SIZE,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        ));
                    });

                // Value display
                controls.spawn((
                    Text::new(format!("{}%", (current_value * 100.0) as u8)),
                    TextFont {
                        font_size: LABEL_FONT_SIZE,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    Node {
                        width: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    UiBrightnessText,
                ));

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
                        UiBrightnessUpButton,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("+"),
                            TextFont {
                                font_size: BUTTON_FONT_SIZE,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        ));
                    });
            });
        });
}

/// Cleans up the settings menu UI when exiting the state.
///
/// Despawns all entities marked with `OnSettingsScreen`.
///
/// # Arguments
///
/// * `commands` - Bevy command buffer for despawning entities
/// * `settings_items` - Query for all entities with the `OnSettingsScreen` marker
pub fn cleanup(mut commands: Commands, settings_items: Query<Entity, With<OnSettingsScreen>>) {
    for entity in &settings_items {
        commands.entity(entity).despawn();
    }
}

/// Handles keyboard input in the settings menu.
///
/// - Escape: Returns to Landing screen
///
/// # Arguments
///
/// * `keyboard` - Keyboard input resource
/// * `next_menu_state` - Resource for transitioning the `MenuState`
pub fn keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_menu_state.set(MenuState::Landing);
    }
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

/// Handles button actions when clicked.
///
/// Processes all button types: Back, VSync mode, and difficulty.
///
/// # Arguments
///
/// * `interactions` - Query for button interactions and actions
/// * `next_menu_state` - Resource for menu state transitions
/// * `game_config` - Mutable game configuration resource
pub fn button_action(
    interactions: Query<(&Interaction, &SettingsButtonAction), Changed<Interaction>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut game_config: ResMut<GameConfig>,
) {
    for (interaction, action) in &interactions {
        if *interaction == Interaction::Pressed {
            match action {
                SettingsButtonAction::Back => {
                    next_menu_state.set(MenuState::Landing);
                }
                SettingsButtonAction::SetVsyncOn => {
                    game_config.vsync = VsyncMode::On;
                }
                SettingsButtonAction::SetVsyncOff => {
                    game_config.vsync = VsyncMode::Off;
                }
                SettingsButtonAction::SetVsyncAdaptive => {
                    game_config.vsync = VsyncMode::Adaptive;
                }
                SettingsButtonAction::SetDifficultyEasy => {
                    game_config.difficulty = Difficulty::Easy;
                }
                SettingsButtonAction::SetDifficultyNormal => {
                    game_config.difficulty = Difficulty::Normal;
                }
                SettingsButtonAction::SetDifficultyHard => {
                    game_config.difficulty = Difficulty::Hard;
                }
            }
        }
    }
}

/// Handles mouse wheel scrolling for the settings menu.
///
/// Uses Bevy's built-in ScrollPosition component and HoverMap to enable scrolling.
///
/// # Arguments
///
/// * `mouse_wheel_events` - Event reader for mouse wheel events
/// * `hover_map` - Map of hovered UI entities
/// * `scrollable_query` - Query for scrollable nodes with ScrollPosition
/// * `parent_query` - Query for parent entities to walk up the hierarchy
pub fn handle_scroll(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    hover_map: Res<bevy::picking::hover::HoverMap>,
    mut scrollable_query: Query<(&mut ScrollPosition, &ComputedNode), With<ScrollableContainer>>,
    parent_query: Query<&ChildOf>,
) {
    const LINE_HEIGHT: f32 = 10.0;
    const PIXEL_SCROLL_MULTIPLIER: f32 = 0.3;

    for event in mouse_wheel_events.read() {
        let dy = match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => -event.y * LINE_HEIGHT,
            bevy::input::mouse::MouseScrollUnit::Pixel => -event.y * PIXEL_SCROLL_MULTIPLIER,
        };

        // Check if we're hovering over the scrollable container or any of its children
        for pointer_map in hover_map.values() {
            for (hovered_entity, _) in pointer_map.iter() {
                // Walk up the hierarchy to find a scrollable container
                let mut current_entity = *hovered_entity;
                loop {
                    if let Ok((mut scroll_position, computed)) =
                        scrollable_query.get_mut(current_entity)
                    {
                        let visible_size = computed.size();
                        let content_size = computed.content_size();
                        let max_scroll = (content_size.y - visible_size.y).max(0.0)
                            * computed.inverse_scale_factor();

                        scroll_position.y = (scroll_position.y + dy).clamp(0.0, max_scroll);
                        break;
                    }

                    // Move to parent
                    if let Ok(parent) = parent_query.get(current_entity) {
                        current_entity = parent.get();
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

/// Handles volume button clicks.
///
/// Adjusts volume levels up or down in 10% increments, clamped to 0.0-1.0.
///
/// # Arguments
///
/// * `down_buttons` - Query for volume decrease buttons
/// * `up_buttons` - Query for volume increase buttons
/// * `user_prefs` - Mutable user preferences resource
pub fn volume_button_action(
    down_buttons: Query<(&Interaction, &VolumeDownButton), Changed<Interaction>>,
    up_buttons: Query<(&Interaction, &VolumeUpButton), Changed<Interaction>>,
    mut game_config: ResMut<GameConfig>,
) {
    const VOLUME_STEP: f32 = 0.01;

    for (interaction, button) in &down_buttons {
        if *interaction == Interaction::Pressed {
            match button.volume_type {
                VolumeType::Master => {
                    game_config.master_volume = (game_config.master_volume - VOLUME_STEP).max(0.0);
                }
                VolumeType::Music => {
                    game_config.music_volume = (game_config.music_volume - VOLUME_STEP).max(0.0);
                }
                VolumeType::Sfx => {
                    game_config.sfx_volume = (game_config.sfx_volume - VOLUME_STEP).max(0.0);
                }
            }
        }
    }

    for (interaction, button) in &up_buttons {
        if *interaction == Interaction::Pressed {
            match button.volume_type {
                VolumeType::Master => {
                    game_config.master_volume = (game_config.master_volume + VOLUME_STEP).min(1.0);
                }
                VolumeType::Music => {
                    game_config.music_volume = (game_config.music_volume + VOLUME_STEP).min(1.0);
                }
                VolumeType::Sfx => {
                    game_config.sfx_volume = (game_config.sfx_volume + VOLUME_STEP).min(1.0);
                }
            }
        }
    }
}

/// Handles UI brightness adjustment button interactions.
pub fn ui_brightness_button_action(
    down_buttons: Query<&Interaction, (Changed<Interaction>, With<UiBrightnessDownButton>)>,
    up_buttons: Query<&Interaction, (Changed<Interaction>, With<UiBrightnessUpButton>)>,
    mut game_config: ResMut<GameConfig>,
) {
    const BRIGHTNESS_STEP: f32 = 0.1;

    for interaction in &down_buttons {
        if *interaction == Interaction::Pressed {
            game_config.brightness = (game_config.brightness - BRIGHTNESS_STEP).max(0.0);
        }
    }

    for interaction in &up_buttons {
        if *interaction == Interaction::Pressed {
            game_config.brightness = (game_config.brightness + BRIGHTNESS_STEP).min(2.0);
        }
    }
}

/// Updates volume text displays when volumes change.
///
/// # Arguments
///
/// * `user_prefs` - User preferences resource
/// * `volume_texts` - Query for volume text components
pub fn update_volume_text(
    game_config: Res<GameConfig>,
    mut volume_texts: Query<(&mut Text, &VolumeText)>,
) {
    if game_config.is_changed() {
        for (mut text, volume_text) in &mut volume_texts {
            let volume = match volume_text.volume_type {
                VolumeType::Master => game_config.master_volume,
                VolumeType::Music => game_config.music_volume,
                VolumeType::Sfx => game_config.sfx_volume,
            };
            text.0 = format!("{}%", (volume * 100.0) as u8);
        }
    }
}

/// Updates volume slider fill widths and handle positions when volumes change.
pub fn update_volume_sliders(
    game_config: Res<GameConfig>,
    mut slider_fills: Query<(&mut Node, &VolumeSliderFill), Without<VolumeSliderHandle>>,
    mut slider_handles: Query<(&mut Node, &VolumeSliderHandle), Without<VolumeSliderFill>>,
) {
    if game_config.is_changed() {
        for (mut node, slider_fill) in &mut slider_fills {
            let volume = match slider_fill.volume_type {
                VolumeType::Master => game_config.master_volume,
                VolumeType::Music => game_config.music_volume,
                VolumeType::Sfx => game_config.sfx_volume,
            };
            node.width = Val::Percent(volume * 100.0);
        }

        for (mut node, slider_handle) in &mut slider_handles {
            let volume = match slider_handle.volume_type {
                VolumeType::Master => game_config.master_volume,
                VolumeType::Music => game_config.music_volume,
                VolumeType::Sfx => game_config.sfx_volume,
            };
            // Center the handle on the position (200px track width, -2px offset for 4px handle)
            node.left = Val::Px(volume * 200.0 - 2.0);
        }
    }
}

/// Handles dragging volume slider handles to set volume directly.
pub fn volume_slider_interaction(
    buttons: Res<ButtonInput<bevy::input::mouse::MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut slider_handles: Query<(&Interaction, &mut VolumeSliderHandle)>,
    mut game_config: ResMut<GameConfig>,
) {
    const SLIDER_WIDTH: f32 = 200.0;

    // Track which handle is being dragged
    for (interaction, mut slider_handle) in &mut slider_handles {
        if *interaction == Interaction::Pressed
            && buttons.pressed(bevy::input::mouse::MouseButton::Left)
        {
            slider_handle.is_dragging = true;
        } else {
            slider_handle.is_dragging = false;
        }
    }

    // Apply mouse delta to dragging handles
    let total_delta: f32 = mouse_motion.read().map(|motion| motion.delta.x).sum();

    if total_delta != 0.0 {
        for (_interaction, slider_handle) in &slider_handles {
            if slider_handle.is_dragging {
                let current_volume = match slider_handle.volume_type {
                    VolumeType::Master => game_config.master_volume,
                    VolumeType::Music => game_config.music_volume,
                    VolumeType::Sfx => game_config.sfx_volume,
                };

                // Convert delta pixels to volume change
                let volume_delta = total_delta / SLIDER_WIDTH;
                let new_volume = (current_volume + volume_delta).clamp(0.0, 1.0);

                match slider_handle.volume_type {
                    VolumeType::Master => game_config.master_volume = new_volume,
                    VolumeType::Music => game_config.music_volume = new_volume,
                    VolumeType::Sfx => game_config.sfx_volume = new_volume,
                }
            }
        }
    }
}

/// Updates UI brightness text display when brightness changes.
pub fn update_ui_brightness_text(
    game_config: Res<GameConfig>,
    mut brightness_texts: Query<&mut Text, With<UiBrightnessText>>,
) {
    if game_config.is_changed() {
        for mut text in &mut brightness_texts {
            text.0 = format!("{}%", (game_config.brightness * 100.0) as u8);
        }
    }
}

/// Updates selected state styling for option buttons.
///
/// Highlights buttons corresponding to current configuration values.
///
/// # Arguments
///
/// * `commands` - Bevy command buffer
/// * `user_prefs` - User preferences resource
/// * `vsync_buttons` - Query for VSync mode buttons
/// * `difficulty_buttons` - Query for difficulty buttons
pub fn update_selected_options(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    mut vsync_buttons: Query<
        (
            Entity,
            &VsyncModeButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
    mut difficulty_buttons: Query<
        (
            Entity,
            &DifficultyButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (With<Button>, Without<VsyncModeButton>),
    >,
) {
    if game_config.is_changed() {
        for (entity, vsync_button, mut bg, mut border) in &mut vsync_buttons {
            if vsync_button.0 == game_config.vsync {
                commands.entity(entity).insert(SelectedOption);
                *bg = BackgroundColor(SELECTED_BACKGROUND);
                *border = BorderColor::all(SELECTED_BORDER);
            } else {
                commands.entity(entity).remove::<SelectedOption>();
                *bg = BackgroundColor(BUTTON_BACKGROUND);
                *border = BorderColor::all(BUTTON_BORDER);
            }
        }

        for (entity, difficulty_button, mut bg, mut border) in &mut difficulty_buttons {
            if difficulty_button.0 == game_config.difficulty {
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
