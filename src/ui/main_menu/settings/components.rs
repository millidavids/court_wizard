//! Settings menu specific components.

use bevy::prelude::*;

use crate::config::{Difficulty, VsyncMode};

/// Marker component for entities that belong to the settings screen.
///
/// Used for cleanup when exiting the settings state.
#[derive(Component)]
pub struct OnSettingsScreen;

/// Marker component for the scrollable container in settings.
#[derive(Component)]
pub struct ScrollableContainer;

/// Button action types for settings menu interactions.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum SettingsButtonAction {
    /// Button to return to the landing screen
    Back,
    /// Button to enable VSync
    SetVsyncOn,
    /// Button to disable VSync
    SetVsyncOff,
    /// Button to enable adaptive VSync
    SetVsyncAdaptive,
    /// Button to set difficulty to Easy
    SetDifficultyEasy,
    /// Button to set difficulty to Normal
    SetDifficultyNormal,
    /// Button to set difficulty to Hard
    SetDifficultyHard,
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

/// Component that tracks which VSync mode a button represents.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct VsyncModeButton(pub VsyncMode);

/// Component that tracks which difficulty a button represents.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct DifficultyButton(pub Difficulty);

/// Component for volume slider value display text.
#[derive(Component)]
pub struct VolumeText {
    /// Which volume this text displays (Master, Music, or SFX)
    pub volume_type: VolumeType,
}

/// Types of volume controls.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VolumeType {
    /// Master volume control
    Master,
    /// Music volume control
    Music,
    /// Sound effects volume control
    Sfx,
}

/// Button to decrease volume.
#[derive(Component)]
pub struct VolumeDownButton {
    /// Which volume to decrease
    pub volume_type: VolumeType,
}

/// Button to increase volume.
#[derive(Component)]
pub struct VolumeUpButton {
    /// Which volume to increase
    pub volume_type: VolumeType,
}

/// Component for volume slider track.
#[derive(Component)]
pub struct VolumeSliderTrack {
    /// Which volume this slider controls
    pub volume_type: VolumeType,
}

/// Component for volume slider fill (the filled portion of the track).
#[derive(Component)]
pub struct VolumeSliderFill {
    /// Which volume this fill represents
    pub volume_type: VolumeType,
}

/// Component for volume slider handle (the draggable knob).
#[derive(Component)]
pub struct VolumeSliderHandle {
    /// Which volume this handle controls
    pub volume_type: VolumeType,
    /// Whether this handle is currently being dragged
    pub is_dragging: bool,
}

/// Component for UI brightness slider value display text.
#[derive(Component)]
pub struct UiBrightnessText;

/// Button to decrease UI brightness.
#[derive(Component)]
pub struct UiBrightnessDownButton;

/// Button to increase UI brightness.
#[derive(Component)]
pub struct UiBrightnessUpButton;
