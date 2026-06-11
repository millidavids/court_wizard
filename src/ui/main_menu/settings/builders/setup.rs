//! Settings screen setup and shared widget helpers.

use bevy::prelude::*;

use crate::config::GameConfig;
use crate::ui::systems::spawn_title_with_shadow;

use super::super::components::{
    ButtonColors, OnSettingsScreen, SelectedOption, SettingsButtonAction, SettingsContentContainer,
    SettingsTab, SettingsTabButton, SettingsTabState, SliderDownButton, SliderFill, SliderHandle,
    SliderText, SliderTrack, SliderUpButton, SliderValue,
};
use super::super::constants::{
    ACTIVE_TAB_BG, ACTIVE_TAB_BORDER, BUTTON_BACKGROUND, BUTTON_BORDER, BUTTON_BORDER_WIDTH,
    BUTTON_FONT_SIZE, INACTIVE_TAB_BG, LABEL_FONT_SIZE, MARGIN, MARGIN_SMALL, OPTION_BUTTON_HEIGHT,
    OPTION_BUTTON_WIDTH, SELECTED_BACKGROUND, SELECTED_BORDER, TAB_BORDER_COLOR, TAB_FONT_SIZE,
    TAB_HEIGHT, TAB_PADDING_H, TEXT_COLOR, TITLE_FONT_SIZE,
};

/// Sets up the settings menu UI with a tabbed interface.
///
/// Creates a settings screen with tabs for Graphics, Audio, Game, and Controls.
/// Tab content is rebuilt dynamically by `rebuild_settings_content` when the
/// active tab changes.
///
/// All spawned entities are marked with `OnSettingsScreen` for cleanup.
fn setup(mut commands: Commands, mut tab_state: ResMut<SettingsTabState>, pause_menu: bool) {
    use crate::ui::systems::spawn_page_container;

    // Reset to default tab when entering settings
    tab_state.active_tab = SettingsTab::Game;

    let content = spawn_page_container(
        &mut commands,
        OnSettingsScreen,
        pause_menu,
        crate::ui::systems::default_content_node(),
    );
    commands.entity(content).with_children(|parent| {
        // Header row: title left, Back button right
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(MARGIN)),
                ..default()
            })
            .with_children(|header| {
                spawn_title_with_shadow(
                    header,
                    "Settings",
                    TITLE_FONT_SIZE,
                    TEXT_COLOR,
                    Node::default(),
                );
                header.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });
                header
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(150.0),
                            height: Val::Px(50.0),
                            border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                        BorderColor::all(BUTTON_BORDER),
                        BackgroundColor(BUTTON_BACKGROUND),
                        ButtonColors {
                            background: BUTTON_BACKGROUND,
                            border: BUTTON_BORDER,
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

        // Tab bar
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(4.0),
                row_gap: Val::Px(4.0),
                margin: UiRect::bottom(Val::Px(MARGIN)),
                ..default()
            })
            .with_children(|tab_bar| {
                for &tab in SettingsTab::all() {
                    let is_active = tab == tab_state.active_tab;
                    let (bg, border) = if is_active {
                        (ACTIVE_TAB_BG, ACTIVE_TAB_BORDER)
                    } else {
                        (INACTIVE_TAB_BG, TAB_BORDER_COLOR)
                    };
                    tab_bar
                        .spawn((
                            Button,
                            Node {
                                height: Val::Px(TAB_HEIGHT),
                                padding: UiRect::horizontal(Val::Px(TAB_PADDING_H)),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(bg),
                            BorderColor::all(border),
                            ButtonColors {
                                background: bg,
                                border,
                            },
                            SettingsTabButton(tab),
                            crate::ui::focus::Focusable,
                            crate::ui::focus::TabFocusable,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(tab.label()),
                                TextFont::from_font_size(TAB_FONT_SIZE),
                                TextColor(TEXT_COLOR),
                            ));
                        });
                }
            });

        // Scrollable content container (rebuilt by rebuild_settings_content)
        parent.spawn((
            crate::ui::systems::scroll_area_style(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                overflow: Overflow::scroll_y(),
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            }),
            ScrollPosition::default(),
            SettingsContentContainer,
            crate::ui::focus::GamepadScrollTarget,
        ));
    });
}

/// Spawns settings with opaque background (for main menu).
pub fn setup_main_menu(commands: Commands, tab_state: ResMut<SettingsTabState>) {
    setup(commands, tab_state, false);
}

/// Spawns settings with transparent background and GlobalZIndex (for pause menu).
pub fn setup_pause_menu(commands: Commands, tab_state: ResMut<SettingsTabState>) {
    setup(commands, tab_state, true);
}

// ---------------------------------------------------------------------------
// Shared widget helpers
// ---------------------------------------------------------------------------

/// Helper function to spawn a row with a label and option buttons.
pub(super) fn spawn_option_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    spawn_buttons: impl FnOnce(&mut ChildSpawnerCommands),
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

            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_shrink: 0.0,
                column_gap: Val::Px(MARGIN_SMALL),
                ..default()
            })
            .with_children(spawn_buttons);
        });
}

/// Spawns a dot-leader filler that expands to fill available horizontal space.
pub(super) fn spawn_dot_leader(parent: &mut ChildSpawnerCommands) {
    crate::ui::systems::spawn_dot_leader(parent, LABEL_FONT_SIZE);
}

/// Helper function to spawn an option button. `action` is any component or
/// bundle that identifies this button to its click handler (e.g.
/// `OptionButtonValue::CrtEnabled(true)` or `ResolutionPreset { width, height }`).
pub(super) fn spawn_option_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    action: impl Bundle,
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
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        },
        BorderColor::all(border_color),
        BackgroundColor(bg_color),
        ButtonColors {
            background: bg_color,
            border: border_color,
        },
        crate::ui::focus::Focusable,
        action,
    ));

    if is_selected {
        entity.insert((SelectedOption, crate::ui::components::ButtonActive));
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

/// Helper function to spawn a slider control row.
pub(super) fn spawn_slider_control(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    slider_value: SliderValue,
    game_config: &GameConfig,
) {
    let current_value = slider_value.get(game_config);

    crate::ui::systems::spawn_slider_row(
        parent,
        crate::ui::systems::SliderRowConfig {
            label,
            current_value,
            min_value: slider_value.min_value(),
            max_value: slider_value.max_value(),
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
