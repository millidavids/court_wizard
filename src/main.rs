use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin, WindowResolution};

mod config;

use config::ConfigPlugin;

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
        .add_plugins(ConfigPlugin::default())
        .insert_resource(ClearColor(Color::srgb(0.2, 0.2, 0.2)))
        .add_systems(Startup, setup)
        .run();
}

/// Sets up the initial game scene.
///
/// Spawns the primary 2D camera required for rendering.
///
/// # Arguments
///
/// * `commands` - Bevy command buffer for spawning entities
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
