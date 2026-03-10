use bevy::prelude::*;
use bevy::window::{CursorLeft, CursorMoved, PrimaryWindow};

use super::components::{ChannelChangeTimer, CrtEffectSettings, DesaturationTimer, LensingSettings};
use super::constants::{CHANNEL_CHANGE_DURATION, DESATURATION_DURATION, LENSING_INFLUENCE_MULT, LENSING_STRENGTH};
use super::messages::{ChannelChangeMessage, ScreenDesaturateMessage};
use crate::game::units::wizard::spells::black_hole::components::BlackHole;

/// Stores the raw (uncorrected) cursor position from OS events.
///
/// Without this, the system would read back its own corrected position
/// from the previous frame and re-apply barrel distortion each frame,
/// causing the cursor to drift toward the screen edge.
#[derive(Resource, Default)]
pub(super) struct RawCursorPosition(pub(super) Option<Vec2>);

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
fn barrel_correct(raw_logical: Vec2, logical_width: f32, logical_height: f32, barrel_distortion: f32) -> Vec2 {
    let uv = Vec2::new(
        raw_logical.x / logical_width,
        raw_logical.y / logical_height,
    );

    let centered = uv - Vec2::new(0.5, 0.5);
    let dist_sq = centered.dot(centered);
    let corrected_uv =
        centered * (1.0 + barrel_distortion * dist_sq) + Vec2::new(0.5, 0.5);

    Vec2::new(
        corrected_uv.x * logical_width,
        corrected_uv.y * logical_height,
    )
}

pub(super) fn correct_cursor_for_barrel_distortion(
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
pub(super) fn correct_ui_interaction_for_barrel(
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
        .filter_map(|(entity, camera)| {
            let viewport_position = camera
                .physical_viewport_rect()
                .map(|rect| rect.min.as_vec2())
                .unwrap_or_default();
            Some((entity, corrected_physical - viewport_position))
        })
        .collect();

    let mouse_clicked = mouse_button_input.just_pressed(MouseButton::Left);

    // Walk the UI stack from top to bottom, same as ui_focus_system
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
        )) = node_query.get_mut(*node_entity)
        else {
            continue;
        };

        let Some(mut interaction) = interaction else {
            continue;
        };

        // Skip invisible nodes
        if inherited_visibility
            .map(|v| v.get())
            != Some(true)
        {
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

        if blocked {
            // Already found a blocking node above — reset lower nodes
            if *interaction != Interaction::Pressed {
                interaction.set_if_neq(Interaction::None);
            }
            continue;
        }

        if contains_cursor {
            if mouse_clicked {
                if *interaction != Interaction::Pressed {
                    *interaction = Interaction::Pressed;
                }
            } else if *interaction == Interaction::None {
                *interaction = Interaction::Hovered;
            }

            // Check focus policy
            match focus_policy.unwrap_or(&bevy::ui::FocusPolicy::Block) {
                bevy::ui::FocusPolicy::Block => {
                    blocked = true;
                }
                bevy::ui::FocusPolicy::Pass => {}
            }
        } else if *interaction != Interaction::Pressed {
            interaction.set_if_neq(Interaction::None);
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

/// Projects active black hole positions to viewport-local UV space for gravitational lensing.
///
/// The CRT shader operates in viewport-local UV (0–1 within the letterboxed region),
/// so we must convert from full-window UV to local UV using the viewport offset/size.
pub(super) fn update_lensing_positions(
    black_holes: Query<&BlackHole>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut lensing_query: Query<&mut LensingSettings>,
) {
    let Ok(mut settings) = lensing_query.single_mut() else {
        return;
    };

    // Reset lensing data
    settings.lensing_count = 0.0;
    settings.lensing_strength = LENSING_STRENGTH;
    settings.lensing_darkening = 0.0;
    settings.lensing_0_x = 0.0;
    settings.lensing_0_y = 0.0;
    settings.lensing_0_radius = 0.0;
    settings.lensing_1_x = 0.0;
    settings.lensing_1_y = 0.0;
    settings.lensing_1_radius = 0.0;

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let mut count = 0u32;
    let mut max_growth: f32 = 0.0;
    for black_hole in &black_holes {
        if count >= 2 {
            break;
        }
        // Track growth for screen darkening (0→1 over GROWTH_TIME)
        let growth = (black_hole.time_alive / crate::game::units::wizard::spells::black_hole::constants::GROWTH_TIME).min(1.0);
        max_growth = max_growth.max(growth);

        // Project black hole center to NDC
        let Some(ndc) = camera.world_to_ndc(camera_transform, black_hole.position) else {
            continue;
        };

        // Convert NDC (-1..1) to full-window UV (0..1), flipping Y for screen space
        let full_uv_x = (ndc.x + 1.0) * 0.5;
        let full_uv_y = 1.0 - (ndc.y + 1.0) * 0.5;

        // Skip if too far off screen
        if full_uv_x < -0.3 || full_uv_x > 1.3 || full_uv_y < -0.3 || full_uv_y > 1.3 {
            continue;
        }

        // Project a point at the edge of the black hole to get screen-space radius
        let edge_point =
            black_hole.position + camera_transform.right() * black_hole.current_radius;
        let Some(edge_ndc) = camera.world_to_ndc(camera_transform, edge_point) else {
            continue;
        };
        let edge_full_uv_x = (edge_ndc.x + 1.0) * 0.5;
        let screen_radius = (edge_full_uv_x - full_uv_x).abs();

        // Influence radius is larger than visual radius
        let influence_radius = screen_radius * LENSING_INFLUENCE_MULT;

        // Skip black holes that are too small (still growing)
        if influence_radius < 0.001 {
            continue;
        }

        match count {
            0 => {
                settings.lensing_0_x = full_uv_x;
                settings.lensing_0_y = full_uv_y;
                settings.lensing_0_radius = influence_radius;
            }
            1 => {
                settings.lensing_1_x = full_uv_x;
                settings.lensing_1_y = full_uv_y;
                settings.lensing_1_radius = influence_radius;
            }
            _ => {}
        }
        count += 1;
    }
    settings.lensing_count = count as f32;
    // Gradual screen darkening: 0→0.3 as black hole grows (70% brightness at full size)
    settings.lensing_darkening = max_growth * 0.3;
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
