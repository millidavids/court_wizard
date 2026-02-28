use bevy::math::DVec2;
use bevy::prelude::*;
use bevy::window::{CursorLeft, CursorMoved};

use super::components::{ChannelChangeTimer, CrtEffectSettings, DesaturationTimer};
use super::constants::{CHANNEL_CHANGE_DURATION, DESATURATION_DURATION};
use super::messages::{ChannelChangeMessage, ScreenDesaturateMessage};

/// Stores the raw (uncorrected) cursor position from OS events.
///
/// Without this, the system would read back its own corrected position
/// from the previous frame and re-apply barrel distortion each frame,
/// causing the cursor to drift toward the screen edge.
#[derive(Resource, Default)]
pub(super) struct RawCursorPosition(Option<Vec2>);

/// Corrects the stored cursor position to account for CRT barrel distortion.
///
/// The CRT shader maps each output pixel at UV `uv` to source content at
/// `barrel_distort(uv)`. When the user's cursor is at screen position P,
/// the content they see came from `barrel_distort(P)` in the undistorted
/// render. Applying the forward barrel distortion to the cursor position
/// makes all downstream systems (spells, UI picking, input messages)
/// automatically target the correct content.
///
/// At screen center `dist_sq = 0`, so no correction is applied.
/// At edges the correction increases quadratically.
pub(super) fn correct_cursor_for_barrel_distortion(
    mut windows: Query<&mut Window>,
    crt_query: Query<&CrtEffectSettings>,
    mut cursor_moved: MessageReader<CursorMoved>,
    mut cursor_left: MessageReader<CursorLeft>,
    mut raw_pos: ResMut<RawCursorPosition>,
) {
    // Track raw cursor position from OS events. CursorMoved provides the
    // true OS position before our correction, avoiding feedback loops.
    for event in cursor_moved.read() {
        raw_pos.0 = Some(event.position);
    }
    if cursor_left.read().last().is_some() {
        raw_pos.0 = None;
    }

    let Ok(settings) = crt_query.single() else {
        return;
    };

    // Skip if CRT is disabled or no barrel distortion
    if settings.enabled < 0.5 || settings.barrel_distortion == 0.0 {
        return;
    }

    let Ok(mut window) = windows.single_mut() else {
        return;
    };

    let Some(raw_logical) = raw_pos.0 else {
        return;
    };

    let logical_width = window.width();
    let logical_height = window.height();

    if logical_width == 0.0 || logical_height == 0.0 {
        return;
    }

    // Normalize to UV (0-1)
    let uv = Vec2::new(raw_logical.x / logical_width, raw_logical.y / logical_height);

    // Apply forward barrel distortion (same formula as the shader)
    let centered = uv - Vec2::new(0.5, 0.5);
    let dist_sq = centered.dot(centered);
    let corrected_uv =
        centered * (1.0 + settings.barrel_distortion * dist_sq) + Vec2::new(0.5, 0.5);

    // Convert corrected UV to physical pixels
    let phys_width = window.physical_width() as f64;
    let phys_height = window.physical_height() as f64;
    let corrected_physical = DVec2::new(
        corrected_uv.x as f64 * phys_width,
        corrected_uv.y as f64 * phys_height,
    );

    window.set_physical_cursor_position(Some(corrected_physical));
}

/// Reads `ChannelChangeMessage` and starts the channel-change animation.
pub(super) fn handle_channel_change_message(
    mut commands: Commands,
    mut messages: MessageReader<ChannelChangeMessage>,
    existing_timer: Option<Res<ChannelChangeTimer>>,
) {
    if messages.read().next().is_some() {
        // Don't restart if already animating
        if existing_timer.is_none() {
            commands.insert_resource(ChannelChangeTimer::new(CHANNEL_CHANGE_DURATION));
        }
    }
}

/// Ticks the channel-change timer, writes intensity to CrtEffectSettings,
/// and removes the timer when the animation is complete.
pub(super) fn animate_channel_change(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<ChannelChangeTimer>,
    mut query: Query<&mut CrtEffectSettings>,
) {
    timer.elapsed += time.delta_secs();

    let intensity = timer.intensity();

    for mut settings in &mut query {
        settings.channel_change = intensity;
        settings.channel_change_time = timer.elapsed;
    }

    if timer.is_finished() {
        // Reset to zero before removing
        for mut settings in &mut query {
            settings.channel_change = 0.0;
            settings.channel_change_time = 0.0;
        }
        commands.remove_resource::<ChannelChangeTimer>();
    }
}

/// Reads `ScreenDesaturateMessage` and starts the desaturation animation.
pub(super) fn handle_desaturation_message(
    mut commands: Commands,
    mut messages: MessageReader<ScreenDesaturateMessage>,
    existing_timer: Option<Res<DesaturationTimer>>,
) {
    if messages.read().next().is_some() && existing_timer.is_none() {
        commands.insert_resource(DesaturationTimer::new(DESATURATION_DURATION));
    }
}

/// Ticks the desaturation timer, writes intensity to CrtEffectSettings,
/// and removes the timer when the animation is complete.
pub(super) fn animate_desaturation(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<DesaturationTimer>,
    mut query: Query<&mut CrtEffectSettings>,
) {
    timer.elapsed += time.delta_secs();

    let intensity = timer.intensity();

    for mut settings in &mut query {
        settings.desaturation = intensity;
    }

    if timer.is_finished() {
        for mut settings in &mut query {
            settings.desaturation = 0.0;
        }
        commands.remove_resource::<DesaturationTimer>();
    }
}
