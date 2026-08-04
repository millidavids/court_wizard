#![cfg_attr(
    all(not(debug_assertions), not(feature = "benchmarking")),
    windows_subsystem = "windows"
)]

use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::camera::{ClearColorConfig, Viewport};
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window, WindowPlugin, WindowResolution};
use bevy::winit::WinitWindows;

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
    ColorblindCorrectionSettings, CrtEffectSettings, HeatDistortionSettings, HighContrastSettings,
    LensingSettings, TeleportDistortionSettings,
};
use game::multiplayer::MultiplayerGamePlugin;
use music::MusicPlugin;
use networking::NetworkingPlugin;
use state::StatePlugin;
use steam::SteamPlugin;
use ui::UiPlugin;

/// On Linux, steer winit toward the X11 (XWayland) backend by default.
///
/// wgpu combined with the NVIDIA proprietary driver frequently fails to create a
/// Vulkan surface on *native* Wayland: `Surface::configure` returns
/// `InvalidSurface` and the game hard-crashes on launch, before the main menu
/// ever appears (this is exactly what a Steam launch hits on such systems).
/// XWayland is dramatically more reliable across GPU vendors and driver
/// versions, so we clear `WAYLAND_DISPLAY` before the winit event loop is built,
/// which makes winit fall back to X11.
///
/// Guardrails:
/// - Only acts when an X server is actually reachable (`DISPLAY` set — XWayland
///   provides this on virtually every Wayland desktop), so we never strand a
///   session that has no X11 fallback.
/// - Skipped entirely under gamescope (Steam Deck Game Mode, and the nested
///   sessions Big Picture uses). That compositor owns the surface and is
///   overwhelmingly AMD, so it never hits the NVIDIA bug this works around —
///   forcing XWayland there only costs a needless translation layer.
/// - Players whose native Wayland stack works well (typically AMD/Intel) can opt
///   back in by launching with `COURT_WIZARD_WAYLAND=1`.
#[cfg(target_os = "linux")]
fn prefer_x11_backend() {
    let opted_into_wayland = std::env::var("COURT_WIZARD_WAYLAND")
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false);
    let on_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    let xserver_available = std::env::var_os("DISPLAY").is_some();
    let under_gamescope = std::env::var_os("GAMESCOPE_WAYLAND_DISPLAY").is_some()
        || std::env::var_os("SteamDeck").is_some()
        || std::env::var("XDG_CURRENT_DESKTOP")
            .is_ok_and(|d| d.to_ascii_lowercase().contains("gamescope"));

    if !opted_into_wayland && !under_gamescope && on_wayland && xserver_available {
        // SAFETY: this runs as the first statement of `main`, before any plugin,
        // thread, or the winit event loop exists, so no other thread can be
        // reading the environment concurrently.
        unsafe { std::env::remove_var("WAYLAND_DISPLAY") };
        eprintln!(
            "[court_wizard] Using X11/XWayland for stability on NVIDIA+Wayland; \
             set COURT_WIZARD_WAYLAND=1 to force native Wayland."
        );
    }
}

#[cfg(not(target_os = "linux"))]
fn prefer_x11_backend() {}

/// Main entry point for the game.
///
/// Initializes the Bevy app with default window settings and the config plugin.
/// The ConfigPlugin will load saved settings from localStorage at startup and
/// apply them to the window.
fn main() {
    // Must run before anything touches winit/the renderer (see the fn docs).
    prefer_x11_backend();

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
                // Resolve assets relative to the executable's directory (or the
                // .app bundle's Contents/Resources on macOS) so the game works
                // regardless of what CWD it's launched from.
                file_path: config::resource_root()
                    .map(|d| d.join("assets").to_string_lossy().into_owned())
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

    // Don't let a stray command targeting an entity that despawned the same
    // frame (e.g. a unit that died mid-tick inside a DoT/AoE zone) crash the
    // player's game. Debug keeps Bevy's default `panic` so real logic bugs
    // still surface during development; release downgrades to a logged warning.
    #[cfg(not(debug_assertions))]
    app.insert_resource(bevy::ecs::error::DefaultErrorHandler(
        bevy::ecs::error::warn,
    ));

    app.add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                enforce_aspect_ratio,
                apply_global_brightness,
                set_window_icon.run_if(run_once),
            ),
        )
        .run();

    // Force the OS process to terminate immediately after Bevy returns.
    // Without this, background threads (notably the iroh transport runtime,
    // which can hold network sockets, and Steam's internal threads) can
    // block the process from exiting — on macOS this freezes the game and
    // the launching terminal for tens of seconds. Saves are flushed in
    // Bevy's `Last` schedule before this point, so it is safe to skip the
    // remaining Drop chain.
    std::process::exit(0);
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
fn setup(
    mut commands: Commands,
    adapter_info: Option<Res<bevy::render::renderer::RenderAdapterInfo>>,
) {
    // Which GPU/backend wgpu picked, in player logs — the Steam overlay hooks
    // the graphics API, so overlay problems can't be diagnosed without this.
    if let Some(info) = adapter_info {
        info!("wgpu adapter: {} ({:?})", info.name, info.backend);
    }

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
        HighContrastSettings::default(),
        ColorblindCorrectionSettings::default(),
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

/// Sets the window icon from the embedded logo image.
///
/// Uses the `image` crate to decode the PNG and `winit` to apply it.
/// If it fails for any reason, the game continues with the default icon.
fn set_window_icon(windows: Option<NonSend<WinitWindows>>) {
    let Some(windows) = windows else {
        warn!("WinitWindows not available, skipping window icon");
        return;
    };
    let icon_bytes = include_bytes!("../assets/images/logos/logo.png");
    let Ok(image) = image::load_from_memory(icon_bytes) else {
        warn!("Failed to decode window icon image");
        return;
    };
    let rgba = image.into_rgba8();
    let (width, height) = rgba.dimensions();
    let Ok(icon) = winit::window::Icon::from_rgba(rgba.into_raw(), width, height) else {
        warn!("Failed to create window icon");
        return;
    };
    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
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
