use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, Mana, Wizard};
use super::components::{WallOfStone, WallOfStoneCaster, WallOfStonePreview};
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::input::events::MouseLeftReleased;

/// Handles Wall of Stone casting — click to anchor, drag to extend, release to place.
#[allow(clippy::too_many_arguments)]
pub fn handle_wall_of_stone_casting(
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (Entity, &Transform, &Wizard, &mut CastingState, &mut Mana),
        With<Wizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut caster_query: Query<&mut WallOfStoneCaster, With<Wizard>>,
    mut preview_query: Query<&mut Transform, (With<WallOfStonePreview>, Without<Wizard>)>,
) {
    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana)) =
        wizard_query.single_mut()
    else {
        return;
    };

    let mut caster = if let Ok(c) = caster_query.single_mut() {
        c
    } else {
        commands
            .entity(wizard_entity)
            .insert(WallOfStoneCaster::new());
        return;
    };

    let mouse_released = mouse_left_released.read().next().is_some();

    // Get cursor world position
    let Some(cursor_pos) = get_cursor_world_position(&camera_query, &window_query) else {
        return;
    };
    let clamped_pos =
        clamp_to_spell_range(cursor_pos, wizard_transform.translation, wizard.spell_range);

    // Handle release — place wall or cancel
    if mouse_released {
        if let Some(anchor) = caster.anchor {
            let diff = Vec3::new(clamped_pos.x - anchor.x, 0.0, clamped_pos.z - anchor.z);
            let length = diff.length();

            if length >= MIN_WALL_LENGTH && mana.can_afford(MANA_COST) {
                let clamped_length = length.min(MAX_WALL_LENGTH);
                let forward = diff.normalize();
                let right = Vec3::new(-forward.z, 0.0, forward.x);
                let center = anchor + forward * (clamped_length / 2.0);

                mana.consume(MANA_COST);

                // Spawn the actual wall
                let wall_mesh = Cuboid::new(clamped_length, WALL_HEIGHT, WALL_WIDTH);
                let rotation = Quat::from_rotation_arc(Vec3::X, forward);

                commands.spawn((
                    Mesh3d(meshes.add(wall_mesh)),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: WALL_COLOR,
                        ..default()
                    })),
                    Transform::from_xyz(center.x, WALL_HEIGHT / 2.0, center.z)
                        .with_rotation(rotation),
                    WallOfStone {
                        center,
                        half_length: clamped_length / 2.0,
                        half_width: WALL_WIDTH / 2.0,
                        forward,
                        right,
                        height: WALL_HEIGHT,
                        time_alive: 0.0,
                        duration: WALL_DURATION,
                        sinking: false,
                    },
                    OnGameplayScreen,
                ));
            }

            // Despawn preview
            if let Some(preview_entity) = caster.preview_entity {
                commands.entity(preview_entity).despawn();
            }

            caster.anchor = None;
            caster.preview_entity = None;
            casting_state.cancel();
            mouse_state.left_consumed = true;
        }
        return;
    }

    match *casting_state {
        CastingState::Resting => {
            if !mana.can_afford(MANA_COST) {
                return;
            }

            // Set anchor and spawn preview
            caster.anchor = Some(clamped_pos);

            let preview_entity = commands
                .spawn((
                    Mesh3d(meshes.add(Cuboid::new(1.0, WALL_HEIGHT, WALL_WIDTH))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: WALL_PREVIEW_COLOR,
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        cull_mode: None,
                        ..default()
                    })),
                    Transform::from_xyz(clamped_pos.x, WALL_HEIGHT / 2.0, clamped_pos.z)
                        .with_scale(Vec3::new(0.0, 1.0, 1.0)),
                    WallOfStonePreview,
                    OnGameplayScreen,
                ))
                .id();

            caster.preview_entity = Some(preview_entity);
            casting_state.start_cast();
        }
        CastingState::Casting { .. } => {
            // Update preview to stretch from anchor to cursor
            if let Some(anchor) = caster.anchor
                && let Some(preview_entity) = caster.preview_entity
                && let Ok(mut preview_transform) = preview_query.get_mut(preview_entity)
            {
                let diff = Vec3::new(clamped_pos.x - anchor.x, 0.0, clamped_pos.z - anchor.z);
                let length = diff.length().min(MAX_WALL_LENGTH);

                if length > 0.1 {
                    let forward = diff.normalize();
                    let center = anchor + forward * (length / 2.0);
                    let rotation = Quat::from_rotation_arc(Vec3::X, forward);

                    preview_transform.translation =
                        Vec3::new(center.x, WALL_HEIGHT / 2.0, center.z);
                    preview_transform.rotation = rotation;
                    preview_transform.scale = Vec3::new(length, 1.0, 1.0);
                }
            }
        }
        _ => {}
    }
}

/// Handles right-click cancellation of wall placement.
pub fn handle_wall_of_stone_cancel(
    mut mouse_right_pressed: MessageReader<crate::game::input::events::MouseRightPressed>,
    mut commands: Commands,
    mut wizard_query: Query<&mut CastingState, With<Wizard>>,
    mut caster_query: Query<&mut WallOfStoneCaster, With<Wizard>>,
    mut mouse_state: ResMut<MouseButtonState>,
) {
    if mouse_right_pressed.read().next().is_none() {
        return;
    }

    let Ok(mut casting_state) = wizard_query.single_mut() else {
        return;
    };

    let Ok(mut caster) = caster_query.single_mut() else {
        return;
    };

    if let Some(preview_entity) = caster.preview_entity {
        commands.entity(preview_entity).despawn();
    }

    caster.anchor = None;
    caster.preview_entity = None;
    casting_state.cancel();
    mouse_state.left_consumed = true;
}

/// Advances wall lifetime and triggers sinking phase.
pub fn tick_wall_lifetime(time: Res<Time>, mut walls: Query<&mut WallOfStone>) {
    let delta = time.delta_secs();
    for mut wall in &mut walls {
        wall.time_alive += delta;
        if !wall.sinking && wall.time_alive >= wall.duration - WALL_SINK_DURATION {
            wall.sinking = true;
        }
    }
}

/// Animates walls sinking into the ground during their final seconds.
pub fn animate_sinking_walls(mut walls: Query<(&WallOfStone, &mut Transform)>) {
    for (wall, mut transform) in &mut walls {
        if wall.sinking {
            let sink_elapsed = wall.time_alive - (wall.duration - WALL_SINK_DURATION);
            let sink_progress = (sink_elapsed / WALL_SINK_DURATION).clamp(0.0, 1.0);
            let target_y = wall.height / 2.0 - wall.height * sink_progress;
            transform.translation.y = target_y;
        }
    }
}

/// Despawns walls that have exceeded their duration.
pub fn cleanup_expired_walls(mut commands: Commands, walls: Query<(Entity, &WallOfStone)>) {
    for (entity, wall) in &walls {
        if wall.time_alive >= wall.duration {
            commands.entity(entity).despawn();
        }
    }
}

/// Gets cursor position projected onto Y=0 plane.
fn get_cursor_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec3> {
    let (camera, camera_transform) = camera_query.single().ok()?;
    let window = window_query.single().ok()?;
    let cursor_pos = window.cursor_position()?;

    let ray = camera
        .viewport_to_world(camera_transform, cursor_pos)
        .ok()?;
    let t = -ray.origin.y / ray.direction.y;

    if t > 0.0 {
        Some(ray.origin + ray.direction * t)
    } else {
        None
    }
}

/// Clamps a position to be within the wizard's spell range.
fn clamp_to_spell_range(target: Vec3, wizard_pos: Vec3, spell_range: f32) -> Vec3 {
    let diff = target - wizard_pos;
    let distance = diff.length();

    if distance > spell_range {
        wizard_pos + diff.normalize() * spell_range
    } else {
        target
    }
}
