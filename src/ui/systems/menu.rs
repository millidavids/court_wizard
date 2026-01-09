use bevy::prelude::*;

use crate::{
    state::MenuState,
    ui::{components::*, styles::*},
};

// Type aliases for complex query types
type MainMenuButtonQuery<'a> = (
    &'a Interaction,
    Option<&'a StartButton>,
    Option<&'a SettingsButton>,
    Option<&'a ExitButton>,
);

type MainMenuButtonFilter = (Changed<Interaction>, With<Button>);
type MainMenuColorFilter = (Changed<Interaction>, With<Button>, With<MainMenuUI>);
type ButtonColorQuery<'a> = (&'a Interaction, &'a mut BackgroundColor);

/// Spawns the main menu UI when entering the StartMenu state or returning from settings
pub fn spawn_main_menu(mut commands: Commands, menu_state: Res<MenuState>) {
    // Only spawn if we're in the Main menu state (not Settings)
    if *menu_state != MenuState::Main {
        return;
    }

    // Root container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            MainMenuUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Main Menu"),
                TextFont {
                    font_size: FONT_SIZE_HEADER,
                    ..default()
                },
                TextColor(HEADER_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ));

            // Start Game button
            parent
                .spawn((
                    Button,
                    button_bundle(),
                    BackgroundColor(NORMAL_BUTTON),
                    StartButton,
                    MainMenuUI,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Start Game"),
                        TextFont {
                            font_size: FONT_SIZE_BUTTON,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
                });

            // Settings button
            parent
                .spawn((
                    Button,
                    button_bundle(),
                    BackgroundColor(NORMAL_BUTTON),
                    SettingsButton,
                    MainMenuUI,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Settings"),
                        TextFont {
                            font_size: FONT_SIZE_BUTTON,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
                });

            // Exit button
            parent
                .spawn((
                    Button,
                    button_bundle(),
                    BackgroundColor(NORMAL_BUTTON),
                    ExitButton,
                    MainMenuUI,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Exit"),
                        TextFont {
                            font_size: FONT_SIZE_BUTTON,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                    ));
                });
        });
}

/// Handles button interactions on the main menu
pub fn handle_main_menu_buttons(
    mut menu_state: ResMut<MenuState>,
    mut app_exit_events: MessageWriter<AppExit>,
    interaction_query: Query<MainMenuButtonQuery, MainMenuButtonFilter>,
) {
    for (interaction, start_btn, settings_btn, exit_btn) in &interaction_query {
        if *interaction == Interaction::Pressed {
            if start_btn.is_some() {
                // Start Game button - no functionality yet
                // TODO: Transition to GameRunning state when game is ready
            } else if settings_btn.is_some() {
                // Settings button - change to settings state (menu transition system handles despawn)
                *menu_state = MenuState::Settings;
            } else if exit_btn.is_some() {
                // Exit button - send app exit event
                app_exit_events.write(AppExit::Success);
            }
        }
    }
}

/// Updates button colors based on interaction state
pub fn update_button_colors(mut interaction_query: Query<ButtonColorQuery, MainMenuColorFilter>) {
    for (interaction, mut color) in &mut interaction_query {
        *color = match *interaction {
            Interaction::Pressed => pressed_color(NORMAL_BUTTON).into(),
            Interaction::Hovered => hovered_color(NORMAL_BUTTON).into(),
            Interaction::None => NORMAL_BUTTON.into(),
        };
    }
}

/// Handles transitions between main menu and settings menu based on MenuState changes
pub fn handle_menu_state_transitions(
    mut commands: Commands,
    menu_state: Res<MenuState>,
    main_menu_query: Query<Entity, (With<MainMenuUI>, Without<ChildOf>)>,
    settings_menu_query: Query<Entity, (With<SettingsMenuUI>, Without<ChildOf>)>,
) {
    if !menu_state.is_changed() {
        return;
    }

    match *menu_state {
        MenuState::Main => {
            // Despawn settings menu root entities (despawn will automatically remove all children)
            for entity in &settings_menu_query {
                commands.entity(entity).despawn();
            }
            // Spawn main menu if it doesn't exist
            if main_menu_query.is_empty() {
                spawn_main_menu(commands, menu_state);
            }
        }
        MenuState::Settings => {
            // Despawn main menu root entities (despawn will automatically remove all children)
            for entity in &main_menu_query {
                commands.entity(entity).despawn();
            }
            // Spawn settings menu if it doesn't exist
            if settings_menu_query.is_empty() {
                super::settings::spawn_settings_menu(commands, menu_state);
            }
        }
    }
}

/// Cleans up all menu UI when exiting the StartMenu state
pub fn cleanup_menu_ui(
    mut commands: Commands,
    main_menu_query: Query<Entity, (With<MainMenuUI>, Without<ChildOf>)>,
    settings_menu_query: Query<Entity, (With<SettingsMenuUI>, Without<ChildOf>)>,
    mut menu_state: ResMut<MenuState>,
) {
    // Despawn all main menu root entities (despawn will automatically remove all children)
    for entity in &main_menu_query {
        commands.entity(entity).despawn();
    }

    // Despawn all settings menu root entities (despawn will automatically remove all children)
    for entity in &settings_menu_query {
        commands.entity(entity).despawn();
    }

    // Reset menu state to Main
    *menu_state = MenuState::Main;
}
