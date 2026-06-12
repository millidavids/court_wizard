use bevy::prelude::*;
use bevy::window::{
    MonitorSelection, PresentMode, PrimaryWindow, Window as BevyWindow, WindowMode, WindowMoved,
    WindowPosition, WindowResized,
};

use super::super::messages::*;
use super::super::resources::*;

/// Applies VSync configuration to Bevy's Window component.
///
/// # Arguments
///
/// * `vsync` - VSync mode from config
/// * `window` - Mutable reference to Bevy's Window component
pub(super) fn apply_vsync_config(vsync: VsyncMode, window: &mut BevyWindow) {
    window.present_mode = match vsync {
        VsyncMode::Off => PresentMode::AutoNoVsync,
        VsyncMode::Adaptive => PresentMode::AutoVsync,
        VsyncMode::On => PresentMode::AutoVsync,
    };

    info!("Applied VSync config: {:?}", vsync);
}

/// Applies display mode configuration to Bevy's Window component.
pub(super) fn apply_display_mode_to_window(display_mode: DisplayMode, window: &mut BevyWindow) {
    window.mode = match display_mode {
        DisplayMode::Windowed => WindowMode::Windowed,
        DisplayMode::BorderlessFullscreen => {
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        }
    };
    info!("Applied display mode: {:?}", display_mode);
}

/// Reactively applies vsync and display mode when the relevant fields change.
///
/// Tracks the previous values to avoid redundant Window mutations (which would
/// cause unnecessary logging and potential flicker) when unrelated GameConfig
/// fields change (e.g. volume sliders).
pub(crate) fn apply_display_mode(
    game_config: Res<GameConfig>,
    mut windows: Query<&mut BevyWindow, With<PrimaryWindow>>,
    mut saved_geometry: ResMut<SavedWindowedGeometry>,
    mut last_vsync: Local<Option<VsyncMode>>,
    mut last_display_mode: Local<Option<DisplayMode>>,
) {
    let vsync_changed = *last_vsync != Some(game_config.vsync);
    let display_changed = *last_display_mode != Some(game_config.display_mode);

    if !vsync_changed && !display_changed {
        return;
    }

    // VSync can be applied immediately (no render target size change)
    if vsync_changed && !display_changed {
        if let Ok(mut window) = windows.single_mut() {
            apply_vsync_config(game_config.vsync, &mut window);
        }
        *last_vsync = Some(game_config.vsync);
    }

    if display_changed {
        let was_windowed = matches!(
            last_display_mode.as_ref(),
            Some(DisplayMode::Windowed) | None
        );

        // Save current windowed geometry before switching away from windowed mode
        if was_windowed
            && game_config.display_mode != DisplayMode::Windowed
            && let Ok(window) = windows.single()
        {
            saved_geometry.width = window.resolution.width();
            saved_geometry.height = window.resolution.height();
            saved_geometry.position = match window.position {
                WindowPosition::At(pos) => Some(pos),
                _ => None,
            };
            info!(
                "Saved windowed geometry: {}x{} at {:?}",
                saved_geometry.width, saved_geometry.height, saved_geometry.position
            );
        }

        // Defer the actual mode change to the next frame to avoid wgpu
        // scissor rect / render target size mismatch crashes.
        saved_geometry.pending_mode_change = Some(game_config.display_mode);
        *last_vsync = Some(game_config.vsync);
        *last_display_mode = Some(game_config.display_mode);
    }
}

/// Applies deferred display mode changes one frame after they were requested.
///
/// This avoids wgpu validation errors where the scissor rect from the previous
/// frame's resolution doesn't fit the new render target size.
pub(crate) fn apply_deferred_mode_change(
    mut saved_geometry: ResMut<SavedWindowedGeometry>,
    game_config: Res<GameConfig>,
    mut windows: Query<&mut BevyWindow, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<Camera3d>>,
) {
    let Some(new_mode) = saved_geometry.pending_mode_change.take() else {
        return;
    };

    let Ok(mut window) = windows.single_mut() else {
        return;
    };

    // Clear the camera viewport before ANY mode change. The OS resizes
    // the window surface asynchronously, so enforce_aspect_ratio may see
    // stale dimensions and set a viewport that doesn't match the actual
    // render target, causing a wgpu scissor rect crash. Setting viewport
    // to None lets the camera use the full surface for one frame until
    // enforce_aspect_ratio recomputes with correct dimensions.
    for mut camera in &mut cameras {
        camera.viewport = None;
    }

    apply_display_mode_to_window(new_mode, &mut window);

    if new_mode == DisplayMode::Windowed {
        window
            .resolution
            .set(saved_geometry.width, saved_geometry.height);
        if let Some(pos) = saved_geometry.position {
            window.position = WindowPosition::At(pos);
        }
        info!(
            "Restored windowed geometry: {}x{} at {:?}",
            saved_geometry.width, saved_geometry.height, saved_geometry.position
        );
    }

    // Always apply current vsync when changing mode
    apply_vsync_config(game_config.vsync, &mut window);
}

/// Detects window resize events and triggers config save.
pub(crate) fn detect_window_resize(
    mut resize_events: MessageReader<WindowResized>,
    mut config_changed: MessageWriter<ConfigChanged>,
) {
    if resize_events.read().count() == 0 {
        return;
    }

    config_changed.write(ConfigChanged);
}

/// Detects window move events and saves the new position to config.
pub(crate) fn detect_window_move(
    mut move_events: MessageReader<WindowMoved>,
    mut config_changed: MessageWriter<ConfigChanged>,
    mut saved_geometry: ResMut<SavedWindowedGeometry>,
    windows: Query<&BevyWindow, With<PrimaryWindow>>,
) {
    if move_events.read().count() == 0 {
        return;
    }

    // Only save if the window is actually visible on screen (ignore minimized/offscreen moves)
    let Ok(window) = windows.single() else {
        return;
    };
    if window.physical_width() == 0 || window.physical_height() == 0 {
        return;
    }

    // Always track the latest position so it's available at exit time
    // even if the window is already closing.
    if let WindowPosition::At(pos) = window.position {
        saved_geometry.position = Some(pos);
    }

    config_changed.write(ConfigChanged);
}

/// Returns the window position, preferring the saved geometry (which tracks
/// every move) over the live window (which may be gone at exit time).
pub(super) fn get_window_position(
    windows: &Query<&BevyWindow, With<PrimaryWindow>>,
    saved_geometry: &SavedWindowedGeometry,
) -> Option<IVec2> {
    // Try live window first
    if let Ok(w) = windows.single()
        && let WindowPosition::At(pos) = w.position
    {
        return Some(pos);
    }
    // Fall back to last saved position
    saved_geometry.position
}
