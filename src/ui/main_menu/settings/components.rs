//! Settings menu specific components.

use bevy::prelude::*;

use crate::config::{Difficulty, VsyncMode};

/// Marker component for entities that belong to the settings screen.
///
/// Used for cleanup when exiting the settings state.
/// This is used by both main menu and pause menu settings.
#[derive(Component)]
pub struct OnSettingsScreen;

/// Marker component for the scrollable container in settings.
#[derive(Component)]
pub struct ScrollableContainer;

/// Identifies which config option a button series controls.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Component)]
pub enum OptionButtonValue {
    /// VSync mode option
    VsyncMode(VsyncMode),
    /// Difficulty option
    Difficulty(Difficulty),
}

impl OptionButtonValue {
    /// Get the current value from GameConfig.
    pub fn is_selected(&self, config: &crate::config::GameConfig) -> bool {
        match self {
            OptionButtonValue::VsyncMode(mode) => config.vsync == *mode,
            OptionButtonValue::Difficulty(difficulty) => config.difficulty == *difficulty,
        }
    }

    /// Set the value in GameConfig.
    pub fn apply(&self, config: &mut crate::config::GameConfig) {
        match self {
            OptionButtonValue::VsyncMode(mode) => config.vsync = *mode,
            OptionButtonValue::Difficulty(difficulty) => config.difficulty = *difficulty,
        }
    }
}

/// Button action types for settings menu interactions.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum SettingsButtonAction {
    /// Button to return to the landing screen
    Back,
}

/// Colors for different button states.
#[derive(Component, Clone, Copy)]
pub struct ButtonColors {
    /// Normal background color
    pub background: Color,
}

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
}

impl SliderValue {
    /// Get the current value from GameConfig.
    pub fn get(&self, config: &crate::config::GameConfig) -> f32 {
        match self {
            SliderValue::MasterVolume => config.master_volume,
            SliderValue::MusicVolume => config.music_volume,
            SliderValue::SfxVolume => config.sfx_volume,
            SliderValue::UiBrightness => config.brightness,
        }
    }

    /// Set the value in GameConfig.
    pub fn set(&self, config: &mut crate::config::GameConfig, value: f32) {
        match self {
            SliderValue::MasterVolume => config.master_volume = value,
            SliderValue::MusicVolume => config.music_volume = value,
            SliderValue::SfxVolume => config.sfx_volume = value,
            SliderValue::UiBrightness => config.brightness = value,
        }
    }

    /// Get the minimum value for this slider.
    pub fn min_value(&self) -> f32 {
        match self {
            SliderValue::MasterVolume | SliderValue::MusicVolume | SliderValue::SfxVolume => 0.0,
            SliderValue::UiBrightness => 0.1, // 10% minimum to prevent soft-lock
        }
    }

    /// Get the maximum value for this slider.
    pub fn max_value(&self) -> f32 {
        match self {
            SliderValue::MasterVolume | SliderValue::MusicVolume | SliderValue::SfxVolume => 1.0,
            SliderValue::UiBrightness => 2.0,
        }
    }

    /// Get the step size for increment/decrement buttons.
    pub fn step(&self) -> f32 {
        match self {
            SliderValue::MasterVolume | SliderValue::MusicVolume | SliderValue::SfxVolume => 0.01,
            SliderValue::UiBrightness => 0.1,
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
