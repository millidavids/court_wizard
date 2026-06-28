//! Per-tab content spawn functions for Graphics, Audio, Accessibility, and Controller tabs.

use bevy::prelude::*;

use crate::config::SavedWindowedGeometry;
use crate::config::{ColorblindType, ControllerGlyphStyle, DisplayMode, GameConfig, VsyncMode};

use super::super::components::{
    ButtonColors, ConfigureControllerButton, OptionButtonValue, ResolutionPreset, ResolutionRow,
    SliderValue,
};
use super::super::constants::{
    BUTTON_BACKGROUND, BUTTON_BORDER, BUTTON_BORDER_WIDTH, BUTTON_FONT_SIZE, LABEL_FONT_SIZE,
    MARGIN_SMALL, OPTION_BUTTON_HEIGHT, OPTION_BUTTON_WIDTH, RESOLUTION_PRESETS, TEXT_COLOR,
};
use super::super::controller_diagrams::spawn_controller_diagram_section;
use super::setup::{spawn_dot_leader, spawn_option_button, spawn_option_row, spawn_slider_control};
use crate::ui::gamepad_glyphs::GamepadGlyphFonts;

/// Spawns Graphics tab content: VSync, Display Mode, Resolution, Brightness.
pub(super) fn spawn_graphics_tab(
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
                    align_items: AlignItems::FlexEnd,
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
                .with_children(|buttons| {
                    for &(w, h, label) in RESOLUTION_PRESETS {
                        let is_selected = (saved_geometry.width - w).abs() < 1.0
                            && (saved_geometry.height - h).abs() < 1.0;
                        spawn_option_button(
                            buttons,
                            label,
                            ResolutionPreset {
                                width: w,
                                height: h,
                            },
                            is_selected,
                        );
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
pub(super) fn spawn_audio_tab(parent: &mut ChildSpawnerCommands, game_config: &GameConfig) {
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

/// Spawns Accessibility tab content: colorblind, high contrast, flash/motion reduction, game speed, aim assist.
pub(super) fn spawn_accessibility_tab(parent: &mut ChildSpawnerCommands, game_config: &GameConfig) {
    let mut wrapper = parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(MARGIN_SMALL),
        ..default()
    });
    wrapper.with_children(|section| {
        spawn_option_row(section, "Aim Assist:", |buttons| {
            spawn_option_button(
                buttons,
                "On",
                OptionButtonValue::AimAssist(true),
                game_config.aim_assist,
            );
            spawn_option_button(
                buttons,
                "Off",
                OptionButtonValue::AimAssist(false),
                !game_config.aim_assist,
            );
        });

        spawn_option_row(section, "CRT Effect:", |buttons| {
            spawn_option_button(
                buttons,
                "On",
                OptionButtonValue::CrtEnabled(true),
                game_config.crt_enabled,
            );
            spawn_option_button(
                buttons,
                "Off",
                OptionButtonValue::CrtEnabled(false),
                !game_config.crt_enabled,
            );
        });

        spawn_option_row(section, "Reduce Flashes:", |buttons| {
            spawn_option_button(
                buttons,
                "On",
                OptionButtonValue::ReduceFlashes(true),
                game_config.reduce_flashes,
            );
            spawn_option_button(
                buttons,
                "Off",
                OptionButtonValue::ReduceFlashes(false),
                !game_config.reduce_flashes,
            );
        });

        spawn_option_row(section, "Reduce Motion:", |buttons| {
            spawn_option_button(
                buttons,
                "On",
                OptionButtonValue::ReduceMotion(true),
                game_config.reduce_motion,
            );
            spawn_option_button(
                buttons,
                "Off",
                OptionButtonValue::ReduceMotion(false),
                !game_config.reduce_motion,
            );
        });

        spawn_option_row(section, "Colorblind Mode:", |buttons| {
            spawn_option_button(
                buttons,
                "None",
                OptionButtonValue::ColorblindMode(ColorblindType::None),
                game_config.colorblind_type == ColorblindType::None,
            );
            spawn_option_button(
                buttons,
                "Protanopia",
                OptionButtonValue::ColorblindMode(ColorblindType::Protanopia),
                game_config.colorblind_type == ColorblindType::Protanopia,
            );
            spawn_option_button(
                buttons,
                "Deuteranopia",
                OptionButtonValue::ColorblindMode(ColorblindType::Deuteranopia),
                game_config.colorblind_type == ColorblindType::Deuteranopia,
            );
            spawn_option_button(
                buttons,
                "Tritanopia",
                OptionButtonValue::ColorblindMode(ColorblindType::Tritanopia),
                game_config.colorblind_type == ColorblindType::Tritanopia,
            );
        });

        spawn_slider_control(
            section,
            "Color Correction:",
            SliderValue::ColorblindStrength,
            game_config,
        );

        spawn_slider_control(
            section,
            "High Contrast:",
            SliderValue::HighContrast,
            game_config,
        );

        spawn_slider_control(section, "Game Speed:", SliderValue::GameSpeed, game_config);
    });
}

/// Spawns Controller tab content: sensitivity / deadzone / response curve sliders + rumble toggle,
/// followed by a controller-binding diagram that follows the active gamepad's vendor style.
pub(super) fn spawn_controller_tab(
    parent: &mut ChildSpawnerCommands,
    game_config: &GameConfig,
    glyph_fonts: Option<&GamepadGlyphFonts>,
    glyph_style: ControllerGlyphStyle,
) {
    let mut wrapper = parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(MARGIN_SMALL),
        ..default()
    });
    wrapper.with_children(|section| {
        spawn_slider_control(
            section,
            "Controller Sensitivity X:",
            SliderValue::GamepadSensitivityX,
            game_config,
        );
        spawn_slider_control(
            section,
            "Controller Sensitivity Y:",
            SliderValue::GamepadSensitivityY,
            game_config,
        );
        spawn_slider_control(
            section,
            "Controller Deadzone:",
            SliderValue::GamepadDeadzone,
            game_config,
        );
        spawn_slider_control(
            section,
            "Controller Response Curve:",
            SliderValue::GamepadResponseCurve,
            game_config,
        );
        spawn_option_row(section, "Controller Rumble:", |buttons| {
            spawn_option_button(
                buttons,
                "On",
                OptionButtonValue::RumbleEnabled(true),
                game_config.rumble_enabled,
            );
            spawn_option_button(
                buttons,
                "Off",
                OptionButtonValue::RumbleEnabled(false),
                !game_config.rumble_enabled,
            );
        });

        // Configure Controller: opens Steam's binding panel (Steam Input only;
        // no-op when running without Steam or no Steam pad is active).
        section
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexEnd,
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("Configure Controller:"),
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
                    ConfigureControllerButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Open"),
                        TextFont::from_font_size(BUTTON_FONT_SIZE),
                        TextColor(TEXT_COLOR),
                    ));
                });
            });

        if let Some(fonts) = glyph_fonts {
            spawn_controller_diagram_section(section, fonts, glyph_style);
        }
    });
}
