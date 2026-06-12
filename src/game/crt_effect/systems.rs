use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::{CursorLeft, CursorMoved, PrimaryWindow};

use super::components::{
    ChannelChangeTimer, CrtEffectSettings, DesaturationTimer, ScreenFlashTimer, VignettePulseTimer,
};
use super::constants::{CHANNEL_CHANGE_DURATION, DESATURATION_DURATION};
use super::messages::{
    ChannelChangeMessage, ScreenDesaturateMessage, ScreenFlashMessage, VignettePulseMessage,
};

/// Converts NDC coordinates (-1..1) to screen UV (0..1), flipping Y for screen space.
pub(super) fn ndc_to_uv(ndc: Vec3) -> Vec2 {
    Vec2::new((ndc.x + 1.0) * 0.5, 1.0 - (ndc.y + 1.0) * 0.5)
}

/// Stores the raw (uncorrected) cursor position from OS events.
///
/// Without this, the system would read back its own corrected position
/// from the previous frame and re-apply barrel distortion each frame,
/// causing the cursor to drift toward the screen edge.
#[derive(Resource, Default)]
pub(crate) struct RawCursorPosition(pub(super) Option<Vec2>);

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
/// Barrel-distortion-corrected cursor position in logical window coordinates.
///
/// All game systems that need the cursor position should read this resource
/// instead of `window.cursor_position()`. This ensures the cursor targets
/// the correct content under CRT barrel distortion.
#[derive(Resource, Default)]
pub(crate) struct CorrectedCursorPosition(pub Option<Vec2>);

/// Applies the barrel distortion formula to a UV coordinate.
fn barrel_correct(
    raw_logical: Vec2,
    logical_width: f32,
    logical_height: f32,
    barrel_distortion: f32,
) -> Vec2 {
    let uv = Vec2::new(
        raw_logical.x / logical_width,
        raw_logical.y / logical_height,
    );

    let centered = uv - Vec2::new(0.5, 0.5);
    let dist_sq = centered.dot(centered);
    let corrected_uv = centered * (1.0 + barrel_distortion * dist_sq) + Vec2::new(0.5, 0.5);

    Vec2::new(
        corrected_uv.x * logical_width,
        corrected_uv.y * logical_height,
    )
}

pub(crate) fn correct_cursor_for_barrel_distortion(
    windows: Query<&Window, With<PrimaryWindow>>,
    crt_query: Query<&CrtEffectSettings>,
    mut cursor_moved: MessageReader<CursorMoved>,
    mut cursor_left: MessageReader<CursorLeft>,
    mut raw_pos: ResMut<RawCursorPosition>,
    mut corrected_pos: ResMut<CorrectedCursorPosition>,
) {
    // Track raw cursor position from OS events. CursorMoved provides the
    // true OS position before our correction, avoiding feedback loops.
    for event in cursor_moved.read() {
        raw_pos.0 = Some(event.position);
    }
    if cursor_left.read().last().is_some() {
        raw_pos.0 = None;
        corrected_pos.0 = None;
        return;
    }

    let Some(raw_logical) = raw_pos.0 else {
        corrected_pos.0 = None;
        return;
    };

    let Ok(settings) = crt_query.single() else {
        corrected_pos.0 = Some(raw_logical);
        return;
    };

    // Skip correction if CRT is disabled or no barrel distortion
    if !settings.is_barrel_active() {
        corrected_pos.0 = Some(raw_logical);
        return;
    }

    let Ok(window) = windows.single() else {
        corrected_pos.0 = Some(raw_logical);
        return;
    };

    let win_w = window.width();
    let win_h = window.height();
    if win_w == 0.0 || win_h == 0.0 {
        corrected_pos.0 = Some(raw_logical);
        return;
    }

    // Viewport bounds in logical pixels (from UV fractions)
    let vp_x = settings.viewport_x * win_w;
    let vp_y = settings.viewport_y * win_h;
    let vp_w = settings.viewport_w * win_w;
    let vp_h = settings.viewport_h * win_h;

    if vp_w == 0.0 || vp_h == 0.0 {
        corrected_pos.0 = Some(raw_logical);
        return;
    }

    // Transform cursor from window-logical space to viewport-local space
    let local = Vec2::new(raw_logical.x - vp_x, raw_logical.y - vp_y);
    let corrected_local = barrel_correct(local, vp_w, vp_h, settings.barrel_distortion);

    // Clamp to viewport bounds (barrel correction can push edge positions outside)
    let clamped_x = corrected_local.x.clamp(0.0, vp_w);
    let clamped_y = corrected_local.y.clamp(0.0, vp_h);

    // Transform back to window-logical space
    corrected_pos.0 = Some(Vec2::new(clamped_x + vp_x, clamped_y + vp_y));
}

/// Corrects UI `Interaction` components after Bevy's `ui_focus_system` has run.
///
/// Bevy's `ui_focus_system` reads `window.physical_cursor_position()` (raw OS
/// cursor) for hit testing, which doesn't account for CRT barrel distortion.
/// We can't modify the Window cursor position (it physically warps the cursor)
/// and we can't modify PointerLocation (`ui_focus_system` ignores it).
///
/// Instead, this system runs AFTER `ui_focus_system` and re-does the hit testing
/// using barrel-corrected cursor coordinates, overriding `Interaction` values.
#[allow(clippy::too_many_arguments)]
pub(crate) fn correct_ui_interaction_for_barrel(
    corrected_cursor: Res<CorrectedCursorPosition>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(Entity, &Camera)>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    ui_stack: Res<bevy::ui::UiStack>,
    mut node_query: Query<(
        Entity,
        &ComputedNode,
        &bevy::ui::ui_transform::UiGlobalTransform,
        Option<&mut Interaction>,
        Option<&bevy::ui::FocusPolicy>,
        Option<&InheritedVisibility>,
        &ComputedUiTargetCamera,
        Option<&mut RelativeCursorPosition>,
    )>,
    clipping_query: Query<(
        &ComputedNode,
        &bevy::ui::ui_transform::UiGlobalTransform,
        &Node,
    )>,
    child_of_query: Query<&ChildOf, Without<bevy::ui::OverrideClip>>,
) {
    let Some(corrected_logical) = corrected_cursor.0 else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    // Convert corrected logical position to physical pixels (matching ui_focus_system)
    let corrected_physical = corrected_logical * window.scale_factor();
    let camera_cursor_positions: Vec<(Entity, Vec2)> = camera_query
        .iter()
        .map(|(entity, camera)| {
            let viewport_position = camera
                .physical_viewport_rect()
                .map(|rect| rect.min.as_vec2())
                .unwrap_or_default();
            (entity, corrected_physical - viewport_position)
        })
        .collect();

    let mouse_down = mouse_button_input.pressed(MouseButton::Left);

    // Walk the UI stack from top to bottom, same as ui_focus_system.
    // This system is FULLY AUTHORITATIVE — it overrides ALL interaction states
    // set by ui_focus_system (which used the uncorrected cursor position).
    let mut blocked = false;
    for node_entity in ui_stack.uinodes.iter().rev() {
        let Ok((
            entity,
            computed_node,
            transform,
            interaction,
            focus_policy,
            inherited_visibility,
            target_camera,
            relative_cursor,
        )) = node_query.get_mut(*node_entity)
        else {
            continue;
        };

        // Skip invisible nodes
        if inherited_visibility.map(|v| v.get()) != Some(true) {
            continue;
        }

        let Some(camera_entity) = target_camera.get() else {
            continue;
        };

        let cursor_pos = camera_cursor_positions
            .iter()
            .find(|(cam, _)| *cam == camera_entity)
            .map(|(_, pos)| *pos);

        let contains_cursor = cursor_pos.is_some_and(|point| {
            computed_node.contains_point(*transform, point)
                && bevy::ui::clip_check_recursive(point, entity, &clipping_query, &child_of_query)
        });

        // Correct RelativeCursorPosition using the barrel-corrected cursor.
        // Always provide a value (even outside bounds) so sliders can clamp
        // when the user drags past the edges.
        if let Some(mut rel_cursor) = relative_cursor {
            if let Some(point) = cursor_pos {
                let node_size = computed_node.size();
                if node_size.x > 0.0 && node_size.y > 0.0 {
                    let node_pos = transform.translation;
                    let half = node_size / 2.0;
                    let min = Vec2::new(node_pos.x - half.x, node_pos.y - half.y);
                    let relative = Vec2::new(
                        (point.x - min.x) / node_size.x - 0.5,
                        (point.y - min.y) / node_size.y - 0.5,
                    );
                    rel_cursor.normalized = Some(relative);
                }
            } else {
                rel_cursor.normalized = None;
            }
        }

        // Non-interactive nodes can still block lower nodes (e.g., overlay panels).
        if let Some(mut interaction) = interaction {
            if blocked {
                interaction.set_if_neq(Interaction::None);
            } else if contains_cursor {
                if mouse_down {
                    interaction.set_if_neq(Interaction::Pressed);
                } else {
                    interaction.set_if_neq(Interaction::Hovered);
                }
            } else {
                interaction.set_if_neq(Interaction::None);
            }
        }

        // Check focus policy for ALL nodes (interactive or not) that contain
        // the cursor, so overlay panels properly block buttons behind them.
        if !blocked && contains_cursor {
            match focus_policy.unwrap_or(&bevy::ui::FocusPolicy::Block) {
                bevy::ui::FocusPolicy::Block => {
                    blocked = true;
                }
                bevy::ui::FocusPolicy::Pass => {}
            }
        }
    }
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

/// Handles an incoming `ScreenFlashMessage`, inserting or replacing the `ScreenFlashTimer`
/// resource so the flash animation plays from the beginning.
///
/// A new flash always takes priority over an in-progress one.
pub(super) fn handle_screen_flash_message(
    mut commands: Commands,
    mut messages: MessageReader<ScreenFlashMessage>,
    existing_timer: Option<Res<ScreenFlashTimer>>,
) {
    if let Some(msg) = messages.read().next() {
        // Replace existing flash (new one takes priority)
        if existing_timer.is_some() {
            commands.remove_resource::<ScreenFlashTimer>();
        }
        commands.insert_resource(ScreenFlashTimer::new(
            msg.color,
            msg.duration,
            msg.intensity,
        ));
    }
}

/// Ticks the screen flash timer, writes color/intensity to CrtEffectSettings.
pub(super) fn animate_screen_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<ScreenFlashTimer>,
    mut query: Query<&mut CrtEffectSettings>,
) {
    timer.elapsed += time.delta_secs();
    let intensity = timer.intensity();

    for mut settings in &mut query {
        settings.screen_flash_r = timer.color[0];
        settings.screen_flash_g = timer.color[1];
        settings.screen_flash_b = timer.color[2];
        settings.screen_flash_intensity = intensity;
    }

    if timer.is_finished() {
        for mut settings in &mut query {
            settings.screen_flash_intensity = 0.0;
        }
        commands.remove_resource::<ScreenFlashTimer>();
    }
}

/// Reads `VignettePulseMessage` and starts the vignette pulse animation.
pub(super) fn handle_vignette_pulse_message(
    mut commands: Commands,
    mut messages: MessageReader<VignettePulseMessage>,
    existing_timer: Option<Res<VignettePulseTimer>>,
) {
    if let Some(msg) = messages.read().next() {
        if existing_timer.is_some() {
            commands.remove_resource::<VignettePulseTimer>();
        }
        commands.insert_resource(VignettePulseTimer::new(msg.duration, msg.intensity));
    }
}

/// Ticks the vignette pulse timer, writes intensity to CrtEffectSettings.
pub(super) fn animate_vignette_pulse(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<VignettePulseTimer>,
    mut query: Query<&mut CrtEffectSettings>,
) {
    timer.elapsed += time.delta_secs();
    let intensity = timer.intensity();

    for mut settings in &mut query {
        settings.vignette_pulse = intensity;
    }

    if timer.is_finished() {
        for mut settings in &mut query {
            settings.vignette_pulse = 0.0;
        }
        commands.remove_resource::<VignettePulseTimer>();
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
