//! Settings menu specific components.

use bevy::prelude::*;

use crate::config::input_bindings::{BindingAction, BindingContext};
use crate::config::{ColorblindType, DisplayMode, VsyncMode};

/// Marker component for entities that belong to the settings screen.
///
/// Used for cleanup when exiting the settings state.
/// This is used by both main menu and pause menu settings.
#[derive(Component)]
pub struct OnSettingsScreen;

/// Identifies which config option a button series controls.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Component)]
pub enum OptionButtonValue {
    /// VSync mode option
    VsyncMode(VsyncMode),
    /// Display mode option (windowed, fullscreen)
    DisplayMode(DisplayMode),
    /// Skip splash screen toggle
    SkipSplash(bool),
    /// Tutorials enabled toggle
    TutorialsEnabled(bool),
    /// Show level clock toggle
    ShowLevelClock(bool),
    /// Colorblind correction mode
    ColorblindMode(ColorblindType),
    /// Reduce screen flashes toggle
    ReduceFlashes(bool),
    /// Reduce motion toggle
    ReduceMotion(bool),
    /// CRT effect toggle
    CrtEnabled(bool),
    /// Auto-pause on focus loss toggle
    AutoPauseOnFocusLoss(bool),
    /// Auto-pause when the Steam overlay opens toggle
    PauseOnSteamOverlay(bool),
    /// Auto-pause when the active controller disconnects toggle
    PauseOnControllerDisconnect(bool),
    /// Aim assist toggle
    AimAssist(bool),
    /// Controller rumble toggle
    RumbleEnabled(bool),
}

impl OptionButtonValue {
    /// Get the current value from GameConfig.
    pub fn is_selected(&self, config: &crate::config::GameConfig) -> bool {
        match self {
            OptionButtonValue::VsyncMode(mode) => config.vsync == *mode,
            OptionButtonValue::DisplayMode(mode) => config.display_mode == *mode,
            OptionButtonValue::SkipSplash(skip) => config.skip_splash == *skip,
            OptionButtonValue::TutorialsEnabled(enabled) => config.tutorials_enabled == *enabled,
            OptionButtonValue::ShowLevelClock(show) => config.show_level_clock == *show,
            OptionButtonValue::ColorblindMode(mode) => config.colorblind_type == *mode,
            OptionButtonValue::ReduceFlashes(v) => config.reduce_flashes == *v,
            OptionButtonValue::ReduceMotion(v) => config.reduce_motion == *v,
            OptionButtonValue::CrtEnabled(v) => config.crt_enabled == *v,
            OptionButtonValue::AutoPauseOnFocusLoss(v) => config.auto_pause_on_focus_loss == *v,
            OptionButtonValue::PauseOnSteamOverlay(v) => config.pause_on_steam_overlay == *v,
            OptionButtonValue::PauseOnControllerDisconnect(v) => {
                config.pause_on_controller_disconnect == *v
            }
            OptionButtonValue::AimAssist(v) => config.aim_assist == *v,
            OptionButtonValue::RumbleEnabled(v) => config.rumble_enabled == *v,
        }
    }

    /// Set the value in GameConfig.
    pub fn apply(&self, config: &mut crate::config::GameConfig) {
        match self {
            OptionButtonValue::VsyncMode(mode) => config.vsync = *mode,
            OptionButtonValue::DisplayMode(mode) => config.display_mode = *mode,
            OptionButtonValue::SkipSplash(skip) => config.skip_splash = *skip,
            OptionButtonValue::TutorialsEnabled(enabled) => config.tutorials_enabled = *enabled,
            OptionButtonValue::ShowLevelClock(show) => config.show_level_clock = *show,
            OptionButtonValue::ColorblindMode(mode) => config.colorblind_type = *mode,
            OptionButtonValue::ReduceFlashes(v) => config.reduce_flashes = *v,
            OptionButtonValue::ReduceMotion(v) => config.reduce_motion = *v,
            OptionButtonValue::CrtEnabled(v) => config.crt_enabled = *v,
            OptionButtonValue::AutoPauseOnFocusLoss(v) => config.auto_pause_on_focus_loss = *v,
            OptionButtonValue::PauseOnSteamOverlay(v) => config.pause_on_steam_overlay = *v,
            OptionButtonValue::PauseOnControllerDisconnect(v) => {
                config.pause_on_controller_disconnect = *v
            }
            OptionButtonValue::AimAssist(v) => config.aim_assist = *v,
            OptionButtonValue::RumbleEnabled(v) => config.rumble_enabled = *v,
        }
    }
}

/// Button action types for settings menu interactions.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum SettingsButtonAction {
    /// Button to return to the landing screen
    Back,
    /// Button to reset tutorial progress
    ResetTutorials,
    /// Button to clear all game progress
    ClearProgress,
    /// Button to reset all key bindings to defaults
    ResetControls,
}

/// Marker for the "Configure Controller" button (opens Steam's binding panel for
/// the active Steam Input controller). Handled by `steam::input`.
#[derive(Component)]
pub(crate) struct ConfigureControllerButton;

/// Re-export shared ButtonColors for settings use.
pub use crate::ui::components::ButtonColors;

/// Marker for currently selected option button.
///
/// Buttons with this component are visually highlighted to indicate
/// the current active setting.
#[derive(Component)]
pub struct SelectedOption;

/// Identifies which config value a slider controls.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Component)]
pub enum SliderValue {
    /// Master volume (0.0-1.0)
    MasterVolume,
    /// Music volume (0.0-1.0)
    MusicVolume,
    /// SFX volume (0.0-1.0)
    SfxVolume,
    /// UI brightness (0.1-2.0, minimum 10% to prevent soft-lock)
    UiBrightness,
    /// Colorblind correction strength (0.0-1.0)
    ColorblindStrength,
    /// Game speed multiplier (0.5-2.0)
    GameSpeed,
    /// High contrast effect strength (0.0-1.0)
    HighContrast,
    /// Gamepad horizontal cursor sensitivity (0.3-2.5)
    GamepadSensitivityX,
    /// Gamepad vertical cursor sensitivity (0.3-2.5)
    GamepadSensitivityY,
    /// Gamepad stick deadzone (0.05-0.30)
    GamepadDeadzone,
    /// Gamepad response curve exponent (1.0-3.5)
    GamepadResponseCurve,
}

impl SliderValue {
    /// Get the current value from GameConfig.
    pub fn get(&self, config: &crate::config::GameConfig) -> f32 {
        match self {
            SliderValue::MasterVolume => config.master_volume,
            SliderValue::MusicVolume => config.music_volume,
            SliderValue::SfxVolume => config.sfx_volume,
            SliderValue::UiBrightness => config.brightness,
            SliderValue::ColorblindStrength => config.colorblind_strength,
            SliderValue::GameSpeed => config.game_speed,
            SliderValue::HighContrast => config.high_contrast_strength,
            SliderValue::GamepadSensitivityX => config.gamepad_sensitivity_x,
            SliderValue::GamepadSensitivityY => config.gamepad_sensitivity_y,
            SliderValue::GamepadDeadzone => config.gamepad_deadzone,
            SliderValue::GamepadResponseCurve => config.gamepad_response_curve,
        }
    }

    /// Set the value in GameConfig.
    pub fn set(&self, config: &mut crate::config::GameConfig, value: f32) {
        match self {
            SliderValue::MasterVolume => config.master_volume = value,
            SliderValue::MusicVolume => config.music_volume = value,
            SliderValue::SfxVolume => config.sfx_volume = value,
            SliderValue::UiBrightness => config.brightness = value,
            SliderValue::ColorblindStrength => config.colorblind_strength = value,
            SliderValue::GameSpeed => config.game_speed = value,
            SliderValue::HighContrast => config.high_contrast_strength = value,
            SliderValue::GamepadSensitivityX => config.gamepad_sensitivity_x = value,
            SliderValue::GamepadSensitivityY => config.gamepad_sensitivity_y = value,
            SliderValue::GamepadDeadzone => config.gamepad_deadzone = value,
            SliderValue::GamepadResponseCurve => config.gamepad_response_curve = value,
        }
    }

    /// Get the minimum value for this slider.
    pub fn min_value(&self) -> f32 {
        match self {
            SliderValue::MasterVolume | SliderValue::MusicVolume | SliderValue::SfxVolume => 0.0,
            SliderValue::UiBrightness => 0.1,
            SliderValue::ColorblindStrength | SliderValue::HighContrast => 0.0,
            SliderValue::GameSpeed => 0.5,
            SliderValue::GamepadSensitivityX | SliderValue::GamepadSensitivityY => 0.3,
            SliderValue::GamepadDeadzone => 0.05,
            SliderValue::GamepadResponseCurve => 1.0,
        }
    }

    /// Get the maximum value for this slider.
    pub fn max_value(&self) -> f32 {
        match self {
            SliderValue::MasterVolume | SliderValue::MusicVolume | SliderValue::SfxVolume => 1.0,
            SliderValue::UiBrightness => 2.0,
            SliderValue::ColorblindStrength | SliderValue::HighContrast => 1.0,
            SliderValue::GameSpeed => 2.0,
            SliderValue::GamepadSensitivityX | SliderValue::GamepadSensitivityY => 2.5,
            SliderValue::GamepadDeadzone => 0.30,
            SliderValue::GamepadResponseCurve => 3.5,
        }
    }

    /// Get the step size for increment/decrement buttons.
    pub fn step(&self) -> f32 {
        match self {
            SliderValue::MasterVolume | SliderValue::MusicVolume | SliderValue::SfxVolume => 0.01,
            SliderValue::UiBrightness => 0.1,
            SliderValue::ColorblindStrength | SliderValue::HighContrast => 0.1,
            SliderValue::GameSpeed => 0.25,
            SliderValue::GamepadSensitivityX | SliderValue::GamepadSensitivityY => 0.1,
            SliderValue::GamepadDeadzone => 0.01,
            SliderValue::GamepadResponseCurve => 0.1,
        }
    }
}

/// Component for slider value display text.
#[derive(Component)]
pub struct SliderText {
    /// Which config value this text displays
    pub value: SliderValue,
}

/// Button to decrease a slider value.
#[derive(Component)]
pub struct SliderDownButton {
    /// Which value to decrease
    pub value: SliderValue,
}

/// Button to increase a slider value.
#[derive(Component)]
pub struct SliderUpButton {
    /// Which value to increase
    pub value: SliderValue,
}

/// Component for slider track.
#[derive(Component)]
pub struct SliderTrack {
    /// Which value this slider controls
    pub value: SliderValue,
}

/// Component for slider fill (the filled portion of the track).
#[derive(Component)]
pub struct SliderFill {
    /// Which value this fill represents
    pub value: SliderValue,
}

/// Component for slider handle (the draggable knob).
#[derive(Component)]
pub struct SliderHandle {
    /// Which value this handle controls
    pub value: SliderValue,
    /// Whether this handle is currently being dragged
    pub is_dragging: bool,
}

/// Marker for the confirmation popup overlay.
#[derive(Component)]
pub struct ConfirmationPopup;

/// Identifies which action the confirmation popup will execute.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationAction {
    /// Confirm button — execute the pending action
    Confirm(SettingsButtonAction),
    /// Cancel button — dismiss the popup
    Cancel,
}

/// Message sent when any slider is adjusted (buttons, drag, or track click).
#[derive(Message)]
pub struct SliderAdjusted;

/// Settings tab categories.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingsTab {
    Graphics,
    Audio,
    #[default]
    Game,
    Controls,
    Controller,
    Accessibility,
}

impl SettingsTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Graphics => "Video",
            Self::Audio => "Audio",
            Self::Game => "Gameplay",
            Self::Controls => "Keybinds",
            Self::Controller => "Controls",
            Self::Accessibility => "Accessibility",
        }
    }

    pub fn all() -> &'static [SettingsTab] {
        &[
            Self::Game,
            Self::Graphics,
            Self::Audio,
            Self::Controller,
            Self::Controls,
            Self::Accessibility,
        ]
    }
}

/// Tracks which settings tab is active.
#[derive(Resource, Default)]
pub(crate) struct SettingsTabState {
    pub active_tab: SettingsTab,
}

/// Identifies a settings tab button.
#[derive(Component)]
pub(crate) struct SettingsTabButton(pub SettingsTab);

/// Marker for the settings content container that gets rebuilt on tab switch.
#[derive(Component)]
pub(crate) struct SettingsContentContainer;

/// Identifies a resolution preset button.
#[derive(Component, Clone, Copy, PartialEq)]
pub(crate) struct ResolutionPreset {
    pub width: f32,
    pub height: f32,
}

/// Marker for the resolution row container so it can be hidden in fullscreen mode.
#[derive(Component)]
pub(crate) struct ResolutionRow;

/// Identifies a rebindable key button.
#[derive(Component, Clone, Copy)]
pub(crate) struct KeyBindingButton {
    pub context: BindingContext,
    pub action: BindingAction,
}

/// Marker for key binding button text that shows the current key.
#[derive(Component, Clone, Copy)]
pub(crate) struct KeyBindingText {
    pub context: BindingContext,
    pub action: BindingAction,
}

/// Marker for the key capture overlay.
#[derive(Component)]
pub(crate) struct KeyCaptureOverlay;

/// State tracking which binding is currently being captured.
#[derive(Resource, Default)]
pub(crate) struct KeyCaptureState {
    pub active: Option<KeyCaptureAction>,
}

/// Tracks the in-progress key capture with optional conflict confirmation.
pub(crate) struct KeyCaptureAction {
    pub context: BindingContext,
    pub action: BindingAction,
    pub pending_conflict: Option<PendingConflict>,
}

/// Tracks a pending conflict that needs user confirmation.
pub(crate) struct PendingConflict {
    pub key: KeyCode,
    pub conflicting_context: BindingContext,
    pub conflicting_action: BindingAction,
}
