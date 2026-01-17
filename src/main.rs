use bevy::prelude::*;
use bevy::render::view::ColorGrading;
use bevy::window::{Window, WindowPlugin, WindowResolution};

mod config;
mod state;
mod ui;

use config::{ConfigPlugin, GameConfig};
use state::StatePlugin;
use ui::UiPlugin;

/// Main entry point for the game.
///
/// Initializes the Bevy app with default window settings and the config plugin.
/// The ConfigPlugin will load saved settings from localStorage at startup and
/// apply them to the window.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "The Game".into(),
                // Default resolution - ConfigPlugin will update at Startup
                resolution: WindowResolution::new(1920, 1080),
                canvas: Some("#bevy-canvas".to_string()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((ConfigPlugin, StatePlugin, UiPlugin))
        .insert_resource(ClearColor(Color::srgb(0.2, 0.2, 0.2)))
        .add_systems(Startup, setup)
        .add_systems(Update, apply_global_brightness)
        .run();
}

/// Sets up the initial game scene.
///
/// Spawns the primary 2D camera with color grading for global brightness control.
///
/// # Arguments
///
/// * `commands` - Bevy command buffer for spawning entities
fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, ColorGrading::default()));
}

/// Applies global brightness setting to all cameras via color grading exposure.
///
/// This system updates camera exposure when the brightness setting changes,
/// affecting everything rendered on screen (UI, game objects, etc.).
///
/// Brightness is mapped to exposure as follows:
/// - brightness 0.0 → exposure -5.0 (very dark)
/// - brightness 1.0 → exposure 0.0 (normal)
/// - brightness 2.0 → exposure 1.0 (brighter)
fn apply_global_brightness(
    config: Res<GameConfig>,
    mut cameras: Query<&mut ColorGrading, With<Camera>>,
) {
    if !config.is_changed() {
        return;
    }

    let brightness = config.brightness.clamp(0.0, 2.0);

    // Map brightness to exposure
    // brightness 0.0-1.0 → exposure -5.0 to 0.0
    // brightness 1.0-2.0 → exposure 0.0 to 1.0
    let exposure = if brightness <= 1.0 {
        (brightness - 1.0) * 5.0
    } else {
        brightness - 1.0
    };

    for mut color_grading in cameras.iter_mut() {
        color_grading.global.exposure = exposure;
    }
}
