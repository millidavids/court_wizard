use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::camera::{ClearColorConfig, Viewport};
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window, WindowPlugin, WindowResolution};

mod config;
mod crash_handler;
mod game;
mod music;
mod networking;
mod state;
mod steam;
mod ui;

use config::{ConfigPlugin, GameConfig};
use game::GamePlugin;
use game::crt_effect::{
    CrtEffectSettings, HeatDistortionSettings, LensingSettings, TeleportDistortionSettings,
};
use game::multiplayer::MultiplayerGamePlugin;
use music::MusicPlugin;
use networking::NetworkingPlugin;
use state::StatePlugin;
use steam::SteamPlugin;
use ui::UiPlugin;

/// Main entry point for the game.
///
/// Initializes the Bevy app with default window settings and the config plugin.
/// The ConfigPlugin will load saved settings from localStorage at startup and
/// apply them to the window.
fn main() {
    crash_handler::install();

    let mut app = App::new();

    // SteamPlugin must be added before DefaultPlugins (before RenderPlugin).
    // Initialization is graceful — if Steam isn't running, the game continues without it.
    app.add_plugins(SteamPlugin);

    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                filter: "wgpu=error,naga=warn".to_string(),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                // Resolve assets relative to the executable's directory
                // so the game works regardless of what CWD it's launched from.
                file_path: std::env::current_exe()
                    .ok()
                    .and_then(|p| {
                        p.parent()
                            .map(|d| d.join("assets").to_string_lossy().into_owned())
                    })
                    .unwrap_or_else(|| "assets".to_string()),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Court Wizard".into(),
                    // Default resolution - ConfigPlugin will update at Startup
                    resolution: WindowResolution::new(1920, 1080),
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
    .insert_resource(ClearColor(Color::BLACK));

    app.add_systems(Startup, setup)
        .add_systems(Update, (enforce_aspect_ratio, apply_global_brightness))
        .run();
}

const TARGET_ASPECT: f32 = 16.0 / 9.0;

/// Enforces a 16:9 aspect ratio using both Camera Viewport and shader uniforms.
///
/// Camera Viewport constrains where the game (3D + UI) renders within the window.
/// Shader viewport bounds tell the CRT shader where the game area is so it can
/// render black outside and confine all effects to the 16:9 region.
fn enforce_aspect_ratio(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&mut Camera, &mut CrtEffectSettings), With<Camera3d>>,
) {
    let Ok(window) = windows.single() else { return };

    let phys_w = window.physical_width();
    let phys_h = window.physical_height();
    if phys_w == 0 || phys_h == 0 {
        return;
    }

    let window_aspect = phys_w as f32 / phys_h as f32;

    let (vp_w, vp_h) = if window_aspect > TARGET_ASPECT {
        let h = phys_h;
        let w = (h as f32 * TARGET_ASPECT) as u32;
        (w, h)
    } else {
        let w = phys_w;
        let h = (w as f32 / TARGET_ASPECT) as u32;
        (w, h)
    };

    let offset_x = (phys_w - vp_w) / 2;
    let offset_y = (phys_h - vp_h) / 2;

    // Shader viewport bounds in UV space (0-1)
    let uv_x = offset_x as f32 / phys_w as f32;
    let uv_y = offset_y as f32 / phys_h as f32;
    let uv_w = vp_w as f32 / phys_w as f32;
    let uv_h = vp_h as f32 / phys_h as f32;

    for (mut camera, mut settings) in &mut camera_query {
        // Set Camera Viewport so game content renders in the 16:9 area
        let needs_viewport_update = match &camera.viewport {
            Some(vp) => {
                vp.physical_position != UVec2::new(offset_x, offset_y)
                    || vp.physical_size != UVec2::new(vp_w, vp_h)
            }
            None => true,
        };
        if needs_viewport_update {
            camera.viewport = Some(Viewport {
                physical_position: UVec2::new(offset_x, offset_y),
                physical_size: UVec2::new(vp_w, vp_h),
                ..default()
            });
        }

        // Set shader viewport bounds so CRT effects are confined
        if (settings.viewport_x - uv_x).abs() > 0.0001
            || (settings.viewport_y - uv_y).abs() > 0.0001
            || (settings.viewport_w - uv_w).abs() > 0.0001
            || (settings.viewport_h - uv_h).abs() > 0.0001
        {
            settings.viewport_x = uv_x;
            settings.viewport_y = uv_y;
            settings.viewport_w = uv_w;
            settings.viewport_h = uv_h;
        }
    }
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
        Camera {
            // Gray background for the CRT game area; ClearColor::BLACK handles
            // the window background outside the viewport.
            clear_color: ClearColorConfig::Custom(Color::srgb(0.2, 0.2, 0.2)),
            ..default()
        },
        Transform::from_xyz(-1000.0, 2500.0, 2500.0) // Zoomed out further back and higher up, shifted left
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y), // Looking at origin
        CrtEffectSettings::default(),
        LensingSettings::default(),
        HeatDistortionSettings::default(),
        TeleportDistortionSettings::default(),
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
