use bevy::{ecs::relationship::Relationship, prelude::*, window::PrimaryWindow};

use crate::{
    config::{Difficulty, GameConfig, SaveConfigEvent},
    state::MenuState,
    ui::{components::*, helper::*, styles::*},
};

// Type aliases for complex query types
type SettingsButtonQuery<'a> = (
    &'a Interaction,
    Option<&'a BackButton>,
    Option<&'a SaveButton>,
    Option<&'a WindowModeButton>,
    Option<&'a AspectRatioButton>,
    Option<&'a ResolutionButton>,
    Option<&'a VsyncButton>,
    Option<&'a DifficultyButton>,
    Option<&'a ScaleFactorButton>,
    Option<&'a MasterVolumeButton>,
    Option<&'a MusicVolumeButton>,
    Option<&'a SfxVolumeButton>,
    Option<&'a AdjustDirection>,
);

type SettingsInfoQuery<'a> = (
    Entity,
    Option<&'a WindowModeButton>,
    Option<&'a AspectRatioButton>,
    Option<&'a ResolutionButton>,
    Option<&'a VsyncButton>,
    Option<&'a DifficultyButton>,
    Option<&'a ScaleFactorButton>,
);

type SettingsButtonFilter = (Changed<Interaction>, With<Button>);
type SettingsColorFilter = (Changed<Interaction>, With<Button>, With<SettingsMenuUI>);
type ButtonColorQuery<'a> = (&'a Interaction, &'a mut BackgroundColor);

/// Spawns the settings menu UI when entering from main menu
pub fn spawn_settings_menu(mut commands: Commands, menu_state: Res<MenuState>) {
    // Only spawn if we're in the Settings menu state
    if *menu_state != MenuState::Settings {
        return;
    }

    // Root container - constrained to window size with scrolling enabled
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition::default(),
            BackgroundColor(BACKGROUND_COLOR),
            SettingsMenuUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Settings"),
                TextFont {
                    font_size: FONT_SIZE_HEADER,
                    ..default()
                },
                TextColor(HEADER_COLOR),
                Node {
                    margin: UiRect::all(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Display Settings Section
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(700.0),
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|section| {
                    // Section header
                    section.spawn((
                        Text::new("Display Settings"),
                        TextFont {
                            font_size: FONT_SIZE_LABEL + 4.0,
                            ..default()
                        },
                        TextColor(HEADER_COLOR),
                        Node {
                            margin: UiRect::bottom(Val::Px(15.0)),
                            ..default()
                        },
                    ));

                    // Window Mode
                    spawn_setting_row(section, "Window Mode:", |row| {
                        row.spawn((
                            Button,
                            settings_button_bundle(),
                            BackgroundColor(NORMAL_BUTTON),
                            WindowModeButton,
                            SettingsMenuUI,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Windowed"),
                                TextFont {
                                    font_size: FONT_SIZE_LABEL,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });
                    });

                    // Aspect Ratio
                    spawn_setting_row(section, "Aspect Ratio:", |row| {
                        row.spawn((
                            Button,
                            settings_button_bundle(),
                            BackgroundColor(NORMAL_BUTTON),
                            AspectRatioButton,
                            SettingsMenuUI,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("16:9"),
                                TextFont {
                                    font_size: FONT_SIZE_LABEL,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });
                    });

                    // Resolution
                    spawn_setting_row(section, "Resolution:", |row| {
                        row.spawn((
                            Button,
                            settings_button_bundle(),
                            BackgroundColor(NORMAL_BUTTON),
                            ResolutionButton,
                            SettingsMenuUI,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("1280x720"),
                                TextFont {
                                    font_size: FONT_SIZE_LABEL,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });
                    });

                    // VSync
                    spawn_setting_row(section, "VSync:", |row| {
                        row.spawn((
                            Button,
                            settings_button_bundle(),
                            BackgroundColor(NORMAL_BUTTON),
                            VsyncButton,
                            SettingsMenuUI,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("On"),
                                TextFont {
                                    font_size: FONT_SIZE_LABEL,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });
                    });

                    // Scale Factor
                    spawn_setting_row(section, "Scale Factor:", |row| {
                        row.spawn((
                            Button,
                            adjust_button_bundle(),
                            BackgroundColor(NORMAL_BUTTON),
                            ScaleFactorButton,
                            AdjustDirection::Decrease,
                            SettingsMenuUI,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("-"),
                                TextFont {
                                    font_size: FONT_SIZE_LABEL,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });

                        row.spawn((
                            Text::new("1.0"),
                            TextFont {
                                font_size: FONT_SIZE_LABEL,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                            Node {
                                width: Val::Px(80.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                        ));

                        row.spawn((
                            Button,
                            adjust_button_bundle(),
                            BackgroundColor(NORMAL_BUTTON),
                            ScaleFactorButton,
                            AdjustDirection::Increase,
                            SettingsMenuUI,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("+"),
                                TextFont {
                                    font_size: FONT_SIZE_LABEL,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });
                    });
                });

            // Audio Settings Section
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(700.0),
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|section| {
                    section.spawn((
                        Text::new("Audio Settings"),
                        TextFont {
                            font_size: FONT_SIZE_LABEL + 4.0,
                            ..default()
                        },
                        TextColor(HEADER_COLOR),
                        Node {
                            margin: UiRect::bottom(Val::Px(15.0)),
                            ..default()
                        },
                    ));

                    // Master Volume
                    spawn_volume_row(section, "Master Volume:", MasterVolumeButton);

                    // Music Volume
                    spawn_volume_row(section, "Music Volume:", MusicVolumeButton);

                    // SFX Volume
                    spawn_volume_row(section, "SFX Volume:", SfxVolumeButton);
                });

            // Game Settings Section
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(700.0),
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|section| {
                    section.spawn((
                        Text::new("Game Settings"),
                        TextFont {
                            font_size: FONT_SIZE_LABEL + 4.0,
                            ..default()
                        },
                        TextColor(HEADER_COLOR),
                        Node {
                            margin: UiRect::bottom(Val::Px(15.0)),
                            ..default()
                        },
                    ));

                    // Difficulty
                    spawn_setting_row(section, "Difficulty:", |row| {
                        row.spawn((
                            Button,
                            settings_button_bundle(),
                            BackgroundColor(NORMAL_BUTTON),
                            DifficultyButton,
                            SettingsMenuUI,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Normal"),
                                TextFont {
                                    font_size: FONT_SIZE_LABEL,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });
                    });
                });

            // Button row (Save and Back)
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(20.0),
                    margin: UiRect::all(Val::Px(30.0)),
                    ..default()
                })
                .with_children(|button_row| {
                    // Save button
                    button_row
                        .spawn((
                            Button,
                            Node {
                                width: BUTTON_WIDTH,
                                height: BUTTON_HEIGHT,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(NORMAL_BUTTON),
                            SaveButton,
                            SettingsMenuUI,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Save"),
                                TextFont {
                                    font_size: FONT_SIZE_BUTTON,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });

                    // Back button
                    button_row
                        .spawn((
                            Button,
                            Node {
                                width: BUTTON_WIDTH,
                                height: BUTTON_HEIGHT,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(NORMAL_BUTTON),
                            BackButton,
                            SettingsMenuUI,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Back"),
                                TextFont {
                                    font_size: FONT_SIZE_BUTTON,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                        });
                });
        });
}

/// Helper function to spawn a setting row with label and control
fn spawn_setting_row<'a>(
    parent: &'a mut ChildSpawnerCommands<'_>,
    label: &str,
    spawn_control: impl FnOnce(&mut ChildSpawnerCommands<'_>) + 'a,
) {
    parent.spawn(setting_row()).with_children(|row| {
        row.spawn((
            Text::new(label),
            TextFont {
                font_size: FONT_SIZE_LABEL,
                ..default()
            },
            TextColor(TEXT_COLOR),
        ));

        spawn_control(row);
    });
}

/// Helper function to spawn a volume control row
fn spawn_volume_row<T: Component + Default + Copy>(
    parent: &mut ChildSpawnerCommands<'_>,
    label: &str,
    marker: T,
) {
    spawn_setting_row(parent, label, |row| {
        row.spawn((
            Button,
            adjust_button_bundle(),
            BackgroundColor(NORMAL_BUTTON),
            marker,
            AdjustDirection::Decrease,
            SettingsMenuUI,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new("-"),
                TextFont {
                    font_size: FONT_SIZE_LABEL,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
        });

        row.spawn((
            Text::new("100%"),
            TextFont {
                font_size: FONT_SIZE_LABEL,
                ..default()
            },
            TextColor(TEXT_COLOR),
            Node {
                width: Val::Px(80.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ));

        row.spawn((
            Button,
            adjust_button_bundle(),
            BackgroundColor(NORMAL_BUTTON),
            marker,
            AdjustDirection::Increase,
            SettingsMenuUI,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new("+"),
                TextFont {
                    font_size: FONT_SIZE_LABEL,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
        });
    });
}

/// Handles button interactions in the settings menu
pub fn handle_settings_buttons(
    mut menu_state: ResMut<MenuState>,
    mut game_config: ResMut<GameConfig>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut save_events: MessageWriter<SaveConfigEvent>,
    interaction_query: Query<SettingsButtonQuery, SettingsButtonFilter>,
) {
    for (
        interaction,
        back_btn,
        save_btn,
        window_mode_btn,
        aspect_ratio_btn,
        resolution_btn,
        vsync_btn,
        difficulty_btn,
        scale_btn,
        master_vol_btn,
        music_vol_btn,
        sfx_vol_btn,
        direction,
    ) in &interaction_query
    {
        if *interaction == Interaction::Pressed {
            if back_btn.is_some() {
                // Back button - return to main menu (menu transition system handles despawn)
                *menu_state = MenuState::Main;
            } else if save_btn.is_some() {
                // Save button - trigger config save message
                save_events.write(SaveConfigEvent);
            } else if window_mode_btn.is_some() {
                // Cycle through window modes
                if let Ok(mut window) = window_query.single_mut() {
                    window.mode = match window.mode {
                        bevy::window::WindowMode::Windowed => {
                            bevy::window::WindowMode::BorderlessFullscreen(
                                bevy::window::MonitorSelection::Primary,
                            )
                        }
                        bevy::window::WindowMode::BorderlessFullscreen(_) => {
                            bevy::window::WindowMode::Fullscreen(
                                bevy::window::MonitorSelection::Primary,
                                bevy::window::VideoModeSelection::Current,
                            )
                        }
                        bevy::window::WindowMode::Fullscreen(_, _) => {
                            bevy::window::WindowMode::Windowed
                        }
                    };
                }
            } else if aspect_ratio_btn.is_some() {
                // Cycle through common aspect ratios by adjusting height, keeping width constant
                if let Ok(mut window) = window_query.single_mut() {
                    let current_width = window.resolution.physical_width();
                    let current_height = window.resolution.physical_height();
                    let current_ratio = current_width as f32 / current_height as f32;

                    // Determine current aspect ratio string
                    let current_aspect_str = match (current_ratio * 100.0).round() as u32 {
                        177 => "16:9",
                        160 => "16:10",
                        133 => "4:3",
                        233 => "21:9",
                        _ => "16:9",
                    };

                    // Get next aspect ratio and convert to decimal
                    let next_aspect_str = next_aspect_ratio(current_aspect_str);
                    let new_aspect = parse_aspect_ratio(next_aspect_str);

                    // Recalculate height based on new aspect ratio, keeping width constant
                    let new_height = (current_width as f32 / new_aspect).round() as u32;

                    // Apply the new resolution to the window
                    window
                        .resolution
                        .set(current_width as f32, new_height as f32);
                }
            } else if resolution_btn.is_some() {
                // Cycle through common resolutions
                if let Ok(mut window) = window_query.single_mut() {
                    let current_width = window.resolution.physical_width();
                    let current_height = window.resolution.physical_height();

                    // Common resolutions - cycle through them
                    let (new_width, new_height) = match (current_width, current_height) {
                        (1280, 720) => (1920, 1080),
                        (1920, 1080) => (2560, 1440),
                        (2560, 1440) => (3840, 2160),
                        (1280, 800) => (1920, 1200),
                        (1920, 1200) => (2560, 1600),
                        (2560, 1600) => (1280, 800),
                        (1024, 768) => (1280, 1024),
                        (1280, 1024) => (1600, 1200),
                        (1600, 1200) => (1024, 768),
                        (2560, 1080) => (3440, 1440),
                        (3440, 1440) => (2560, 1080),
                        _ => (1280, 720), // default
                    };

                    // Apply the new resolution to the window
                    window.resolution.set(new_width as f32, new_height as f32);
                }
            } else if vsync_btn.is_some() {
                // Cycle through VSync modes (only using modes that are widely supported)
                if let Ok(mut window) = window_query.single_mut() {
                    window.present_mode = match window.present_mode {
                        bevy::window::PresentMode::Fifo => bevy::window::PresentMode::Immediate,
                        bevy::window::PresentMode::Immediate => bevy::window::PresentMode::Fifo,
                        _ => bevy::window::PresentMode::Fifo,
                    };
                }
            } else if difficulty_btn.is_some() {
                // Cycle through difficulty levels
                game_config.difficulty = match game_config.difficulty {
                    Difficulty::Easy => Difficulty::Normal,
                    Difficulty::Normal => Difficulty::Hard,
                    Difficulty::Hard => Difficulty::Easy,
                };
            } else if let Some(dir) = direction {
                if scale_btn.is_some() {
                    // Adjust scale factor
                    if let Ok(mut window) = window_query.single_mut() {
                        let current = window.resolution.scale_factor();
                        let delta = 0.1;
                        let new_scale = match dir {
                            AdjustDirection::Increase => (current + delta).min(2.0),
                            AdjustDirection::Decrease => (current - delta).max(0.5),
                        };
                        window.resolution.set_scale_factor(new_scale);
                    }
                } else if master_vol_btn.is_some() {
                    // Adjust master volume (stored in GameConfig for now as config doesn't have audio runtime)
                    // Note: Audio system not implemented yet, but we can still modify the serialized value
                    // This is a placeholder - actual audio implementation would modify audio resources
                } else if music_vol_btn.is_some() {
                    // Placeholder for music volume
                } else if sfx_vol_btn.is_some() {
                    // Placeholder for SFX volume
                }
            }
        }
    }
}

/// Updates the settings UI to reflect current configuration values
pub fn update_settings_ui(
    game_config: Res<GameConfig>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut text_query: Query<(&mut Text, &ChildOf)>,
    button_query: Query<SettingsInfoQuery, With<Button>>,
) {
    let Ok(window) = window_query.single() else {
        return;
    };

    // Update button text based on current values
    for (
        entity,
        window_mode_btn,
        aspect_ratio_btn,
        resolution_btn,
        vsync_btn,
        difficulty_btn,
        scale_btn,
    ) in &button_query
    {
        if window_mode_btn.is_some() {
            // Update window mode button text
            let mode_text = match window.mode {
                bevy::window::WindowMode::Windowed => "Windowed",
                bevy::window::WindowMode::BorderlessFullscreen(_) => "Borderless",
                bevy::window::WindowMode::Fullscreen(_, _) => "Fullscreen",
            };
            update_button_text(&mut text_query, entity, mode_text);
        } else if aspect_ratio_btn.is_some() {
            // Update aspect ratio button text
            let width = window.resolution.physical_width();
            let height = window.resolution.physical_height();
            let ratio = width as f32 / height as f32;

            let aspect_text = match (ratio * 100.0).round() as u32 {
                177 => "16:9",
                160 => "16:10",
                133 => "4:3",
                233 => "21:9",
                _ => "16:9", // default
            };
            update_button_text(&mut text_query, entity, aspect_text);
        } else if resolution_btn.is_some() {
            // Update resolution button text
            let width = window.resolution.physical_width();
            let height = window.resolution.physical_height();
            let res_text = format!("{}x{}", width, height);
            update_button_text(&mut text_query, entity, &res_text);
        } else if vsync_btn.is_some() {
            // Update VSync button text
            let vsync_text = match window.present_mode {
                bevy::window::PresentMode::Fifo => "On",
                bevy::window::PresentMode::Immediate => "Off",
                _ => "On",
            };
            update_button_text(&mut text_query, entity, vsync_text);
        } else if difficulty_btn.is_some() {
            // Update difficulty button text
            let difficulty_text = match game_config.difficulty {
                Difficulty::Easy => "Easy",
                Difficulty::Normal => "Normal",
                Difficulty::Hard => "Hard",
            };
            update_button_text(&mut text_query, entity, difficulty_text);
        } else if scale_btn.is_some() {
            // Update scale factor display - find the text node between the buttons
            // This is handled separately below
        }
    }

    // Update scale factor display text
    for (mut text, parent) in &mut text_query {
        // Check if this text is the scale factor display (not inside a button)
        let parent_has_scale_btn = button_query
            .iter()
            .any(|(e, _, _, _, _, _, scale)| e == parent.get() && scale.is_some());

        if !parent_has_scale_btn && **text == "1.0" {
            // This is likely the scale factor display
            **text = format!("{:.1}", window.resolution.scale_factor());
        }
    }
}

/// Helper function to update button text
fn update_button_text(
    text_query: &mut Query<(&mut Text, &ChildOf)>,
    button_entity: Entity,
    new_text: &str,
) {
    for (mut text, parent) in text_query {
        if parent.get() == button_entity {
            **text = new_text.to_string();
            break;
        }
    }
}

/// Updates button colors based on interaction state
pub fn update_button_colors(mut interaction_query: Query<ButtonColorQuery, SettingsColorFilter>) {
    for (interaction, mut color) in &mut interaction_query {
        *color = match *interaction {
            Interaction::Pressed => pressed_color(NORMAL_BUTTON).into(),
            Interaction::Hovered => hovered_color(NORMAL_BUTTON).into(),
            Interaction::None => NORMAL_BUTTON.into(),
        };
    }
}
