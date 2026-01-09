use bevy::prelude::*;

/// Marker component for all UI entities in the main menu
#[derive(Component)]
pub struct MainMenuUI;

/// Marker component for all UI entities in the settings menu
#[derive(Component)]
pub struct SettingsMenuUI;

/// Marker component for the "Start Game" button
#[derive(Component)]
pub struct StartButton;

/// Marker component for the "Settings" button
#[derive(Component)]
pub struct SettingsButton;

/// Marker component for the "Exit" button
#[derive(Component)]
pub struct ExitButton;

/// Marker component for the "Back" button in settings menu
#[derive(Component)]
pub struct BackButton;

/// Marker component for the "Save" button in settings menu
#[derive(Component)]
pub struct SaveButton;

// Settings control markers

/// Marker for window mode cycle button
#[derive(Component)]
pub struct WindowModeButton;

/// Marker for vsync cycle button
#[derive(Component)]
pub struct VsyncButton;

/// Marker for difficulty cycle button
#[derive(Component)]
pub struct DifficultyButton;

/// Marker for master volume buttons
#[derive(Component, Default, Copy, Clone)]
pub struct MasterVolumeButton;

/// Marker for music volume buttons
#[derive(Component, Default, Copy, Clone)]
pub struct MusicVolumeButton;

/// Marker for SFX volume buttons
#[derive(Component, Default, Copy, Clone)]
pub struct SfxVolumeButton;

/// Marker for scale factor buttons
#[derive(Component)]
pub struct ScaleFactorButton;

/// Marker for aspect ratio buttons
#[derive(Component)]
pub struct AspectRatioButton;

/// Marker for resolution buttons
#[derive(Component)]
pub struct ResolutionButton;

/// Direction for increment/decrement buttons
#[derive(Component)]
pub enum AdjustDirection {
    Increase,
    Decrease,
}
