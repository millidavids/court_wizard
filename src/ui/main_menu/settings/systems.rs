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
    ButtonColors, ConfirmationAction, ConfirmationPopup, KeyBindingButton, KeyBindingText,
    KeyCaptureAction, KeyCaptureOverlay, KeyCaptureState, OnSettingsScreen, OptionButtonValue,
    PendingConflict, ResolutionPreset, ResolutionRow, SelectedOption, SettingsButtonAction,
    SettingsContentContainer, SettingsTab, SettingsTabButton, SettingsTabState, SliderAdjusted,
    SliderDownButton, SliderFill, SliderHandle, SliderText, SliderTrack, SliderUpButton,
    SliderValue,
};
use super::constants::{
    ACTIVE_TAB_BG, ACTIVE_TAB_BORDER, BUTTON_BACKGROUND, BUTTON_BORDER, BUTTON_BORDER_WIDTH,
    BUTTON_FONT_SIZE, DANGER_BUTTON_BACKGROUND, DANGER_BUTTON_BORDER, INACTIVE_TAB_BG,
    LABEL_FONT_SIZE, LOCKED_TEXT_COLOR, LOCKED_TITLE_COLOR, MARGIN, MARGIN_SMALL,
    OPTION_BUTTON_HEIGHT, OPTION_BUTTON_WIDTH, POPUP_BOX_BG, POPUP_OVERLAY_BG, RESOLUTION_PRESETS,
    SECTION_FONT_SIZE, SELECTED_BACKGROUND, SELECTED_BORDER, TAB_BORDER_COLOR, TAB_FONT_SIZE,
    TAB_HEIGHT, TAB_PADDING_H, TEXT_COLOR, TITLE_FONT_SIZE,
};
use crate::config::SavedWindowedGeometry;
use crate::config::input_bindings::{
    BindingAction, BindingContext, is_bindable_key, key_display_name, key_name,
};
use bevy::window::PrimaryWindow;

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
    tab_state.active_tab = SettingsTab::Graphics;

    let content = spawn_page_container(
        &mut commands,
        OnSettingsScreen,
        pause_menu,
        crate::ui::systems::default_content_node(),
    );
    commands.entity(content).with_children(|parent| {
        // Title
        spawn_title_with_shadow(
            parent,
            "Settings",
            TITLE_FONT_SIZE,
            TEXT_COLOR,
            Node {
                margin: UiRect::bottom(Val::Px(MARGIN)),
                ..default()
            },
        );

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
                                ..default()
                            },
                            BackgroundColor(bg),
                            BorderColor::all(border),
                            BorderRadius::all(Val::Px(4.0)),
                            ButtonColors { background: bg },
                            SettingsTabButton(tab),
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
            SettingsContentContainer,
        ));

        // Back button (outside content area)
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
pub fn setup_main_menu(commands: Commands, tab_state: ResMut<SettingsTabState>) {
    setup(commands, tab_state, false);
}

/// Spawns settings with transparent background and GlobalZIndex (for pause menu).
pub fn setup_pause_menu(commands: Commands, tab_state: ResMut<SettingsTabState>) {
    setup(commands, tab_state, true);
}

// ---------------------------------------------------------------------------
// Per-tab content spawn functions
// ---------------------------------------------------------------------------

/// Spawns Graphics tab content: VSync, Display Mode, Resolution, Brightness.
fn spawn_graphics_tab(
    parent: &mut ChildSpawnerCommands,
    game_config: &GameConfig,
    saved_geometry: &SavedWindowedGeometry,
) {
    let mut wrapper = parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(MARGIN_SMALL),
        ..default()
    });
    wrapper.with_children(|section| {
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

        // Resolution (Windowed mode only)
        section
            .spawn((
                ResolutionRow,
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(MARGIN),
                    ..default()
                },
                if game_config.display_mode == DisplayMode::BorderlessFullscreen {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                },
            ))
            .with_children(|row| {
                row.spawn((
                    Text::new("Resolution:"),
                    TextFont::from_font_size(LABEL_FONT_SIZE),
                    TextColor(TEXT_COLOR),
                    Node {
                        width: Val::Px(200.0),
                        ..default()
                    },
                ));
                row.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(MARGIN_SMALL),
                    ..default()
                })
                .with_children(|buttons| {
                    for &(w, h, label) in RESOLUTION_PRESETS {
                        let is_selected = (saved_geometry.width - w).abs() < 1.0
                            && (saved_geometry.height - h).abs() < 1.0;
                        let (bg_color, border_color) = if is_selected {
                            (SELECTED_BACKGROUND, SELECTED_BORDER)
                        } else {
                            (BUTTON_BACKGROUND, BUTTON_BORDER)
                        };
                        let mut entity = buttons.spawn((
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
                            ResolutionPreset {
                                width: w,
                                height: h,
                            },
                        ));
                        if is_selected {
                            entity.insert(SelectedOption);
                        }
                        entity.with_children(|button| {
                            let font_size = crate::ui::systems::scale_font_by_text_width(
                                label.len() as f32,
                                6.0,
                                12.0,
                                0.7,
                                BUTTON_FONT_SIZE,
                            );
                            button.spawn((
                                Text::new(label),
                                TextFont::from_font_size(font_size),
                                TextColor(TEXT_COLOR),
                            ));
                        });
                    }
                });
            });

        spawn_slider_control(
            section,
            "Brightness:",
            SliderValue::UiBrightness,
            game_config,
        );
    });
}

/// Spawns Audio tab content: Master, Music, SFX volume sliders.
fn spawn_audio_tab(parent: &mut ChildSpawnerCommands, game_config: &GameConfig) {
    let mut wrapper = parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(MARGIN_SMALL),
        ..default()
    });
    wrapper.with_children(|section| {
        spawn_slider_control(
            section,
            "Master Volume:",
            SliderValue::MasterVolume,
            game_config,
        );
        spawn_slider_control(
            section,
            "Music Volume:",
            SliderValue::MusicVolume,
            game_config,
        );
        spawn_slider_control(section, "SFX Volume:", SliderValue::SfxVolume, game_config);
    });
}

/// Spawns Game tab content: toggles + reset/clear buttons.
fn spawn_game_tab(parent: &mut ChildSpawnerCommands, game_config: &GameConfig) {
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
}

/// Spawns Controls tab content: key binding subsections + Reset Controls button.
/// Locked wizard archetypes show joke text instead of keybindings.
fn spawn_controls_tab(parent: &mut ChildSpawnerCommands, bindings: &crate::config::InputBindings) {
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
                    "Rune Caster",
                    "Try pressing some keys on your keyboard...",
                );
            }

            if is_unlocked("Battlemage") {
                spawn_controls_subsection(
                    section,
                    "Swordcerer",
                    bindings,
                    &[
                        (
                            "Forward:",
                            BindingContext::Battlemage,
                            BindingAction::MoveForward,
                        ),
                        (
                            "Backward:",
                            BindingContext::Battlemage,
                            BindingAction::MoveBackward,
                        ),
                        ("Left:", BindingContext::Battlemage, BindingAction::MoveLeft),
                        (
                            "Right:",
                            BindingContext::Battlemage,
                            BindingAction::MoveRight,
                        ),
                    ],
                );
            } else {
                spawn_locked_subsection(
                    section,
                    "Swordcerer",
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
                            "Range -:",
                            BindingContext::ArcanoRouter,
                            BindingAction::RangeDown,
                        ),
                        (
                            "Mana +:",
                            BindingContext::ArcanoRouter,
                            BindingAction::ManaUp,
                        ),
                        (
                            "Mana -:",
                            BindingContext::ArcanoRouter,
                            BindingAction::ManaDown,
                        ),
                        (
                            "Power +:",
                            BindingContext::ArcanoRouter,
                            BindingAction::PowerUp,
                        ),
                        (
                            "Power -:",
                            BindingContext::ArcanoRouter,
                            BindingAction::PowerDown,
                        ),
                        (
                            "Speed +:",
                            BindingContext::ArcanoRouter,
                            BindingAction::SpeedUp,
                        ),
                        (
                            "Speed -:",
                            BindingContext::ArcanoRouter,
                            BindingAction::SpeedDown,
                        ),
                    ],
                );
            } else {
                spawn_locked_subsection(
                    section,
                    "Arcanorouter",
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
                    "Meteorologist",
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
                spawn_locked_subsection(section, "Warglock", "Spells are overrated. Find out why.");
            }

            // Reset Controls button
            section
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(MARGIN),
                    margin: UiRect::top(Val::Px(MARGIN)),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("Reset Controls:"),
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

fn spawn_key_binding_row(
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
            align_items: AlignItems::Center,
            column_gap: Val::Px(MARGIN),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
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

/// Opens key capture overlay when a binding button is clicked.
/// Opens key capture overlay when a binding button is clicked.
pub fn key_binding_button_action(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&KeyBindingButton>,
    mut capture_state: ResMut<KeyCaptureState>,
    capture_query: Query<&KeyCaptureOverlay>,
) {
    if !capture_query.is_empty() {
        button_clicked.read();
        return;
    }
    for event in button_clicked.read() {
        if let Ok(binding) = button_query.get(event.button) {
            capture_state.active = Some(KeyCaptureAction {
                context: binding.context,
                action: binding.action,
                pending_conflict: None,
            });
            spawn_capture_overlay(&mut commands);
        }
    }
}

/// Captures key input for rebinding, with unbind and conflict confirmation support.
pub fn capture_key_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut capture_state: ResMut<KeyCaptureState>,
    overlay_query: Query<Entity, With<KeyCaptureOverlay>>,
    mut bindings: ResMut<crate::config::InputBindings>,
) {
    let Some(ref mut action) = capture_state.active else {
        return;
    };
    let context = action.context;
    let action_id = action.action;

    // Escape cancels
    if keyboard.just_pressed(KeyCode::Escape) {
        capture_state.active = None;
        for entity in &overlay_query {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Backspace unbinds
    if keyboard.just_pressed(KeyCode::Backspace) {
        bindings.set(context, action_id, None);
        capture_state.active = None;
        for entity in &overlay_query {
            commands.entity(entity).despawn();
        }
        return;
    }

    let pressed_key = keyboard.get_just_pressed().find(|k| is_bindable_key(**k));
    let Some(&key) = pressed_key else {
        return;
    };

    // Check if confirming a pending conflict (same key pressed again)
    if let Some(ref pending) = action.pending_conflict {
        if pending.key == key {
            // User confirmed — swap: unbind conflicting, bind new
            bindings.set(
                pending.conflicting_context,
                pending.conflicting_action,
                None,
            );
            bindings.set(context, action_id, Some(key));
            capture_state.active = None;
            for entity in &overlay_query {
                commands.entity(entity).despawn();
            }
            return;
        }
        // Different key — clear pending, fall through to check this new key
    }

    // Check for conflict
    if let Some(conflict_display) = bindings.would_conflict(key, context, action_id) {
        if let Some((conf_ctx, conf_action)) = bindings.find_conflict(key, context, action_id) {
            // Enter confirmation mode
            let key_label = key_name(key);
            action.pending_conflict = Some(PendingConflict {
                key,
                conflict_display: conflict_display.clone(),
                conflicting_context: conf_ctx,
                conflicting_action: conf_action,
            });
            // Respawn overlay with warning
            for entity in &overlay_query {
                commands.entity(entity).despawn();
            }
            spawn_capture_overlay_with_warning(
                &mut commands,
                &format!(
                    "{key_label} is already bound to {conflict_display}.\nPress {key_label} again to swap, or press another key."
                ),
            );
        }
        return;
    }

    // No conflict — apply immediately
    bindings.set(context, action_id, Some(key));
    capture_state.active = None;
    for entity in &overlay_query {
        commands.entity(entity).despawn();
    }
}

/// Returns true when no key capture is active (used as run condition for escape navigation).
pub fn key_capture_inactive(capture_state: Res<KeyCaptureState>) -> bool {
    capture_state.active.is_none()
}

fn spawn_capture_overlay(commands: &mut Commands) {
    spawn_capture_overlay_inner(commands, None);
}

fn spawn_capture_overlay_with_warning(commands: &mut Commands, warning: &str) {
    spawn_capture_overlay_inner(commands, Some(warning));
}

fn spawn_capture_overlay_inner(commands: &mut Commands, warning: Option<&str>) {
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
            KeyCaptureOverlay,
            OnSettingsScreen,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(30.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(MARGIN_SMALL),
                        ..default()
                    },
                    BackgroundColor(POPUP_BOX_BG),
                    BorderColor::all(SELECTED_BORDER),
                    BorderRadius::all(Val::Px(8.0)),
                ))
                .with_children(|popup| {
                    if let Some(warning) = warning {
                        popup.spawn((
                            Text::new(warning),
                            TextFont::from_font_size(LABEL_FONT_SIZE),
                            TextColor(Color::srgb(1.0, 0.3, 0.3)),
                        ));
                    } else {
                        popup.spawn((
                            Text::new("Press a key..."),
                            TextFont::from_font_size(SECTION_FONT_SIZE),
                            TextColor(TEXT_COLOR),
                        ));
                    }
                    popup.spawn((
                        Text::new("(Backspace to unbind, Escape to cancel)"),
                        TextFont::from_font_size(LABEL_FONT_SIZE),
                        TextColor(Color::hsla(0.0, 0.0, 0.5, 1.0)),
                    ));
                });
        });
}

/// Updates binding button text when InputBindings changes.
pub fn update_key_binding_text(
    bindings: Res<crate::config::InputBindings>,
    mut texts: Query<(&mut Text, &KeyBindingText)>,
) {
    if !bindings.is_changed() {
        return;
    }
    for (mut text, binding_text) in &mut texts {
        let key = bindings.get(binding_text.context, binding_text.action);
        text.0 = key_display_name(key).to_string();
    }
}

/// Handles resolution preset button clicks.
/// Clears the camera viewport before resizing to avoid wgpu scissor rect crashes.
pub fn resolution_button_action(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&ResolutionPreset>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<Camera3d>>,
    mut saved_geometry: ResMut<SavedWindowedGeometry>,
    game_config: Res<GameConfig>,
) {
    if game_config.display_mode != DisplayMode::Windowed {
        button_clicked.read();
        return;
    }
    for event in button_clicked.read() {
        if let Ok(preset) = button_query.get(event.button) {
            if let Ok(mut window) = windows.single_mut() {
                for mut camera in &mut cameras {
                    camera.viewport = None;
                }
                window.resolution.set(preset.width, preset.height);
                saved_geometry.width = preset.width;
                saved_geometry.height = preset.height;
            }
        }
    }
}

/// Updates resolution button highlighting based on current window geometry.
pub fn update_resolution_selection(
    mut commands: Commands,
    saved_geometry: Res<SavedWindowedGeometry>,
    mut preset_buttons: Query<(
        Entity,
        &ResolutionPreset,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    if !saved_geometry.is_changed() {
        return;
    }
    for (entity, preset, mut bg, mut border) in &mut preset_buttons {
        let matches = (preset.width - saved_geometry.width).abs() < 1.0
            && (preset.height - saved_geometry.height).abs() < 1.0;
        if matches {
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

/// Shows/hides the resolution row based on display mode.
pub fn update_resolution_visibility(
    game_config: Res<GameConfig>,
    mut resolution_row: Query<&mut Visibility, With<ResolutionRow>>,
) {
    if !game_config.is_changed() {
        return;
    }
    for mut vis in &mut resolution_row {
        *vis = if game_config.display_mode == DisplayMode::Windowed {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

// ---------------------------------------------------------------------------
// Tab interaction systems
// ---------------------------------------------------------------------------

/// Handles settings tab button clicks.
pub fn handle_settings_tab_click(
    mut button_clicked: MessageReader<MouseClicked>,
    tab_query: Query<&SettingsTabButton>,
    mut state: ResMut<SettingsTabState>,
) {
    for event in button_clicked.read() {
        if let Ok(tab_btn) = tab_query.get(event.button)
            && state.active_tab != tab_btn.0
        {
            state.active_tab = tab_btn.0;
        }
    }
}

/// Rebuilds the settings content area when the active tab changes.
/// Also updates tab button styling.
#[allow(clippy::too_many_arguments)]
pub fn rebuild_settings_content(
    mut commands: Commands,
    state: Res<SettingsTabState>,
    game_config: Res<GameConfig>,
    saved_geometry: Res<SavedWindowedGeometry>,
    bindings: Res<crate::config::InputBindings>,
    container_query: Query<Entity, With<SettingsContentContainer>>,
    tab_buttons: Query<(Entity, &SettingsTabButton)>,
    mut tab_colors: Query<(&mut BackgroundColor, &mut BorderColor, &mut ButtonColors)>,
) {
    if !state.is_changed() {
        return;
    }

    // Update tab button styling
    for (entity, tab_btn) in &tab_buttons {
        let is_active = tab_btn.0 == state.active_tab;
        let (bg, border) = if is_active {
            (ACTIVE_TAB_BG, ACTIVE_TAB_BORDER)
        } else {
            (INACTIVE_TAB_BG, TAB_BORDER_COLOR)
        };
        if let Ok((mut bg_color, mut border_color, mut colors)) = tab_colors.get_mut(entity) {
            *bg_color = BackgroundColor(bg);
            *border_color = BorderColor::all(border);
            colors.background = bg;
        }
    }

    // Rebuild content
    let Ok(container) = container_query.single() else {
        return;
    };
    commands.entity(container).despawn_related::<Children>();
    commands
        .entity(container)
        .with_children(|parent| match state.active_tab {
            SettingsTab::Graphics => spawn_graphics_tab(parent, &game_config, &saved_geometry),
            SettingsTab::Audio => spawn_audio_tab(parent, &game_config),
            SettingsTab::Game => spawn_game_tab(parent, &game_config),
            SettingsTab::Controls => spawn_controls_tab(parent, &bindings),
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

/// Helper function to spawn a slider control row.
fn spawn_slider_control(
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
            label_width: 200.0,
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
#[allow(clippy::too_many_arguments)]
pub fn settings_button_action(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&SettingsButtonAction>,
    popup_query: Query<&ConfirmationPopup>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
    mut bindings: ResMut<crate::config::InputBindings>,
) {
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
                    spawn_confirmation_popup(&mut commands, *action, "Reset all tutorials?");
                }
                SettingsButtonAction::ClearProgress => {
                    spawn_confirmation_popup(
                        &mut commands,
                        *action,
                        "Clear all progress? This cannot be undone.",
                    );
                }
                SettingsButtonAction::ResetControls => {
                    *bindings = crate::config::InputBindings::default();
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
    mut bindings: ResMut<crate::config::InputBindings>,
) {
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
                    spawn_confirmation_popup(&mut commands, *action, "Reset all tutorials?");
                }
                SettingsButtonAction::ClearProgress => {
                    spawn_confirmation_popup(
                        &mut commands,
                        *action,
                        "Clear all progress? This cannot be undone.",
                    );
                }
                SettingsButtonAction::ResetControls => {
                    *bindings = crate::config::InputBindings::default();
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
            let normalized = (value - min) / range;
            node.left = Val::Px(normalized * crate::ui::constants::SLIDER_TRACK_WIDTH - 2.0);
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

        if is_dragging && let Some(pos) = cursor_pos.normalized {
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
