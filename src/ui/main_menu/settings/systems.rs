//! Settings menu systems.

use bevy::ecs::relationship::Relationship;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::config::{Difficulty, GameConfig, VsyncMode};
use crate::state::{MenuState, PauseMenuState};
use crate::ui::styles::{item_hovered, item_pressed};

/// Marker component to track that a button was pressed down.
#[derive(Component)]
pub(crate) struct ButtonPressedDown;

use super::components::{
    ButtonColors, OnSettingsScreen, OptionButtonValue, ScrollableContainer, SelectedOption,
    SettingsButtonAction, SliderDownButton, SliderFill, SliderHandle, SliderText, SliderTrack,
    SliderUpButton, SliderValue,
};
use super::constants::{
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
                    });

                    // Audio Settings Section
                    spawn_section(parent, "Audio", |section| {
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
                        spawn_slider_control(
                            section,
                            "SFX Volume:",
                            SliderValue::SfxVolume,
                            &game_config,
                        );
                    });

                    // Display Settings Section
                    spawn_section(parent, "Display", |section| {
                        spawn_slider_control(
                            section,
                            "Brightness:",
                            SliderValue::UiBrightness,
                            &game_config,
                        );
                    });

                    // Game Settings Section
                    spawn_section(parent, "Game", |section| {
                        spawn_option_row(section, "Difficulty:", |buttons| {
                            spawn_option_button(
                                buttons,
                                "Easy",
                                OptionButtonValue::Difficulty(Difficulty::Easy),
                                game_config.difficulty == Difficulty::Easy,
                            );
                            spawn_option_button(
                                buttons,
                                "Normal",
                                OptionButtonValue::Difficulty(Difficulty::Normal),
                                game_config.difficulty == Difficulty::Normal,
                            );
                            spawn_option_button(
                                buttons,
                                "Hard",
                                OptionButtonValue::Difficulty(Difficulty::Hard),
                                game_config.difficulty == Difficulty::Hard,
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

/// Handles keyboard input in the settings menu from main menu.
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

/// Handles keyboard input in the settings menu from pause menu.
///
/// - Escape: Returns to pause menu main screen
///
/// # Arguments
///
/// * `keyboard` - Keyboard input resource
/// * `next_pause_menu_state` - Resource for transitioning the `PauseMenuState`
pub fn pause_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_pause_menu_state: ResMut<NextState<PauseMenuState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_pause_menu_state.set(PauseMenuState::Main);
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

/// Handles settings button actions when clicked from main menu.
pub fn settings_button_action(
    mut commands: Commands,
    interactions: Query<
        (
            Entity,
            &Interaction,
            &SettingsButtonAction,
            Option<&ButtonPressedDown>,
        ),
        Changed<Interaction>,
    >,
    mut next_menu_state: ResMut<NextState<MenuState>>,
) {
    for (entity, interaction, action, pressed_down) in &interactions {
        match *interaction {
            Interaction::Pressed => {
                commands.entity(entity).insert(ButtonPressedDown);
            }
            Interaction::Hovered => {
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();

                    match action {
                        SettingsButtonAction::Back => {
                            next_menu_state.set(MenuState::Landing);
                        }
                    }
                }
            }
            Interaction::None => {
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();
                }
            }
        }
    }
}

/// Handles settings button actions when clicked from pause menu.
pub fn pause_settings_button_action(
    mut commands: Commands,
    interactions: Query<
        (
            Entity,
            &Interaction,
            &SettingsButtonAction,
            Option<&ButtonPressedDown>,
        ),
        Changed<Interaction>,
    >,
    mut next_pause_menu_state: ResMut<NextState<PauseMenuState>>,
) {
    for (entity, interaction, action, pressed_down) in &interactions {
        match *interaction {
            Interaction::Pressed => {
                commands.entity(entity).insert(ButtonPressedDown);
            }
            Interaction::Hovered => {
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();

                    match action {
                        SettingsButtonAction::Back => {
                            next_pause_menu_state.set(PauseMenuState::Main);
                        }
                    }
                }
            }
            Interaction::None => {
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();
                }
            }
        }
    }
}

/// Handles option button clicks.
pub fn option_button_action(
    mut commands: Commands,
    interactions: Query<
        (
            Entity,
            &Interaction,
            &OptionButtonValue,
            Option<&ButtonPressedDown>,
        ),
        Changed<Interaction>,
    >,
    mut game_config: ResMut<GameConfig>,
) {
    for (entity, interaction, value, pressed_down) in &interactions {
        match *interaction {
            Interaction::Pressed => {
                commands.entity(entity).insert(ButtonPressedDown);
            }
            Interaction::Hovered => {
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();
                    value.apply(&mut game_config);
                }
            }
            Interaction::None => {
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();
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

/// Handles slider button clicks for increment/decrement.
pub fn slider_button_action(
    mut commands: Commands,
    down_buttons: Query<
        (
            Entity,
            &Interaction,
            &SliderDownButton,
            Option<&ButtonPressedDown>,
        ),
        Changed<Interaction>,
    >,
    up_buttons: Query<
        (
            Entity,
            &Interaction,
            &SliderUpButton,
            Option<&ButtonPressedDown>,
        ),
        Changed<Interaction>,
    >,
    mut game_config: ResMut<GameConfig>,
) {
    for (entity, interaction, button, pressed_down) in &down_buttons {
        match *interaction {
            Interaction::Pressed => {
                commands.entity(entity).insert(ButtonPressedDown);
            }
            Interaction::Hovered => {
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();

                    let current = button.value.get(&game_config);
                    let step = button.value.step();
                    let min = button.value.min_value();
                    let new_value = (current - step).max(min);
                    button.value.set(&mut game_config, new_value);
                }
            }
            Interaction::None => {
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();
                }
            }
        }
    }

    for (entity, interaction, button, pressed_down) in &up_buttons {
        match *interaction {
            Interaction::Pressed => {
                commands.entity(entity).insert(ButtonPressedDown);
            }
            Interaction::Hovered => {
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();

                    let current = button.value.get(&game_config);
                    let step = button.value.step();
                    let max = button.value.max_value();
                    let new_value = (current + step).min(max);
                    button.value.set(&mut game_config, new_value);
                }
            }
            Interaction::None => {
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();
                }
            }
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
pub fn slider_interaction(
    buttons: Res<ButtonInput<bevy::input::mouse::MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut slider_handles: Query<(&Interaction, &mut SliderHandle)>,
    slider_tracks: Query<(&Interaction, &RelativeCursorPosition, &SliderTrack)>,
    mut game_config: ResMut<GameConfig>,
) {
    const SLIDER_WIDTH: f32 = 200.0;

    // Check if track was clicked (jump to position and start dragging)
    if buttons.just_pressed(bevy::input::mouse::MouseButton::Left) {
        for (interaction, cursor_pos, track) in &slider_tracks {
            // Check if track or its children are being interacted with
            if matches!(*interaction, Interaction::Pressed | Interaction::Hovered)
                && let Some(pos) = cursor_pos.normalized
            {
                // RelativeCursorPosition.normalized has center at (0,0)
                // So left edge = -0.5, right edge = 0.5
                // Convert to 0-1 range by adding 0.5
                let normalized = (pos.x + 0.5).clamp(0.0, 1.0);

                // Scale to the appropriate range for this value
                let min = track.value.min_value();
                let max = track.value.max_value();
                let range = max - min;
                let new_value = (min + normalized * range).clamp(min, max);
                track.value.set(&mut game_config, new_value);

                // Start dragging the corresponding handle
                for (_handle_interaction, mut slider_handle) in &mut slider_handles {
                    if slider_handle.value == track.value {
                        slider_handle.is_dragging = true;
                    }
                }
            }
        }
    }

    // Track which handle is being dragged (for direct handle clicks)
    for (interaction, mut slider_handle) in &mut slider_handles {
        if *interaction == Interaction::Pressed
            && buttons.pressed(bevy::input::mouse::MouseButton::Left)
        {
            slider_handle.is_dragging = true;
        } else if !buttons.pressed(bevy::input::mouse::MouseButton::Left) {
            slider_handle.is_dragging = false;
        }
    }

    // Apply mouse delta to dragging handles
    let total_delta: f32 = mouse_motion.read().map(|motion| motion.delta.x).sum();

    if total_delta != 0.0 {
        for (_interaction, slider_handle) in &slider_handles {
            if slider_handle.is_dragging {
                let current = slider_handle.value.get(&game_config);
                let min = slider_handle.value.min_value();
                let max = slider_handle.value.max_value();
                let range = max - min;

                // Convert delta pixels to value change
                let value_delta = (total_delta / SLIDER_WIDTH) * range;
                let new_value = (current + value_delta).clamp(min, max);

                slider_handle.value.set(&mut game_config, new_value);
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
