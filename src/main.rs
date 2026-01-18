use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin, WindowResolution};

mod config;
mod game;
mod state;
mod ui;

use config::{ConfigPlugin, GameConfig};
use game::GamePlugin;
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
        .add_plugins((ConfigPlugin, StatePlugin, UiPlugin, GamePlugin))
        .insert_resource(ClearColor(Color::srgb(0.2, 0.2, 0.2)))
        .add_systems(Startup, setup)
        .add_systems(Update, apply_global_brightness)
        .run();
}

/// Marker component for the brightness overlay.
#[derive(Component)]
struct BrightnessOverlay;

/// Sets up the initial game scene.
///
/// Spawns the primary 3D perspective camera positioned above the castle
/// looking toward the horizon, and brightness overlay.
///
/// # Arguments
///
/// * `commands` - Bevy command buffer for spawning entities
fn setup(mut commands: Commands) {
    // 3D perspective camera pulled way back to see the entire battlefield
    // We can adjust this later once everything is positioned correctly
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2000.0, 2000.0) // Far back and high up
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y), // Looking at origin
    ));

    // Spawn brightness overlay (a fullscreen node that adjusts screen brightness)
    commands.spawn((
        BrightnessOverlay,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::BLACK.with_alpha(0.0)),
        GlobalZIndex(1000), // On top of everything
        Pickable::IGNORE,   // Don't block pointer events
    ));
}

/// Applies global brightness setting via overlay opacity.
///
/// This system updates the brightness overlay when the brightness setting changes.
/// Uses a black overlay with varying opacity to darken the screen, or a white overlay
/// to brighten it.
///
/// Brightness mapping:
/// - brightness 0.1 → black overlay at 90% opacity (darkest, minimum to prevent soft-lock)
/// - brightness 1.0 → no overlay (normal)
/// - brightness 2.0 → white overlay at 50% opacity (brightest)
fn apply_global_brightness(
    config: Res<GameConfig>,
    mut overlay: Query<&mut BackgroundColor, With<BrightnessOverlay>>,
) {
    if !config.is_changed() {
        return;
    }

    let brightness = config.brightness.clamp(0.1, 2.0);

    if let Ok(mut bg) = overlay.single_mut() {
        if brightness < 1.0 {
            // Darken: black overlay with alpha based on how far below 1.0
            // At 0.1 brightness, alpha = 0.9 (90% dark)
            let alpha = 1.0 - brightness;
            *bg = BackgroundColor(Color::BLACK.with_alpha(alpha));
        } else if brightness > 1.0 {
            // Brighten: white overlay with alpha based on how far above 1.0
            let alpha = (brightness - 1.0) * 0.5; // Max 50% white overlay at brightness 2.0
            *bg = BackgroundColor(Color::WHITE.with_alpha(alpha));
        } else {
            // Normal brightness: transparent overlay
            *bg = BackgroundColor(Color::BLACK.with_alpha(0.0));
        }
    }
}
