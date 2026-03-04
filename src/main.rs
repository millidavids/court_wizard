use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin, WindowResolution};

mod config;
mod game;
mod music;
mod networking;
mod state;
mod ui;

use config::{ConfigPlugin, GameConfig};
use game::GamePlugin;
use game::crt_effect::CrtEffectSettings;
use game::multiplayer::MultiplayerGamePlugin;
use music::MusicPlugin;
use networking::NetworkingPlugin;
use state::StatePlugin;
use ui::UiPlugin;

/// Main entry point for the game.
///
/// Initializes the Bevy app with default window settings and the config plugin.
/// The ConfigPlugin will load saved settings from localStorage at startup and
/// apply them to the window.
fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                // Suppress bevy_winit::system "could not set cursor position" errors
                // on WASM — browsers don't support programmatic cursor repositioning,
                // but we still call it (works on native) for CRT barrel correction.
                filter: "bevy_winit::system=off,wgpu=error,naga=warn".to_string(),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Court Wizard".into(),
                    // Default resolution - ConfigPlugin will update at Startup
                    resolution: WindowResolution::new(1920, 1080).with_scale_factor_override(1.0),
                    canvas: Some("#bevy-canvas".to_string()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    .add_plugins((
        ConfigPlugin,
        StatePlugin,
        NetworkingPlugin,
        MusicPlugin,
        UiPlugin,
        GamePlugin,
        MultiplayerGamePlugin,
    ))
    .insert_resource(ClearColor(Color::srgb(0.2, 0.2, 0.2)));

    app.add_systems(Startup, setup)
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
        Transform::from_xyz(-1000.0, 2500.0, 2500.0) // Zoomed out further back and higher up, shifted left
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y), // Looking at origin
        CrtEffectSettings::default(),
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
