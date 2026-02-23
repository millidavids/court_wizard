use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, Mana, PrimedSpell, Spell, LocalWizard, Wizard, WizardInput};
use super::components::{WallOfStone, WallOfStoneCaster, WallOfStonePreview};
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::multiplayer::spell_commands::SkipSpellSpawning;
use crate::networking::protocol::{NetworkMessage, SpellAction};
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::SpellEffectKind;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleType};

/// Result from spell casting logic, used to communicate state back to the wrapper.
struct CastResult {
    /// Whether the spell completed (wall was placed).
    completed: bool,
    /// Whether preview should be despawned (release with too-short drag or no mana).
    despawn_preview: bool,
    /// Anchor position for drag spell network message.
    anchor: Option<Vec3>,
    /// End position for drag spell network message.
    end_pos: Option<Vec3>,
}

/// Local wizard Wall of Stone casting — reads mouse input, manages preview.
///
/// On the guest (when `SkipSpellSpawning` is present), the casting pipeline
/// runs normally but wall spawning is skipped. A `DragSpellCast` message
/// is sent to the host with anchor + end position.
#[allow(clippy::too_many_arguments)]
pub fn handle_wall_of_stone_casting(
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (
            Entity,
            &Transform,
            &Wizard,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut caster_query: Query<&mut WallOfStoneCaster>,
    mut preview_query: Query<&mut Transform, (With<WallOfStonePreview>, Without<Wizard>)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    skip_spawning: Option<Res<SkipSpellSpawning>>,
    mut connection: Option<ResMut<NetworkConnection>>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true, // Run conditions ensure mouse is held
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::WallOfStone {
        return;
    }

    let mut caster = if let Ok(c) = caster_query.get_mut(wizard_entity) {
        c
    } else {
        commands
            .entity(wizard_entity)
            .insert(WallOfStoneCaster::new());
        return;
    };

    let clamped_pos = input.cursor_pos.map(|pos| {
        clamp_to_spell_range(pos, wizard_transform.translation, wizard.spell_range)
    });

    let skip_spawn = skip_spawning.is_some();

    let cast_result = wall_of_stone_casting_logic(
        &input,
        clamped_pos,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut caster,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut obstacle_events,
        skip_spawn,
    );

    // Local-only: manage preview
    match *casting_state {
        CastingState::Resting => {}
        _ => {}
    }

    // Handle preview spawning on cast start
    if caster.anchor.is_some() && caster.preview_entity.is_none() {
        if let Some(pos) = clamped_pos {
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
                    Transform::from_xyz(pos.x, WALL_HEIGHT / 2.0, pos.z)
                        .with_scale(Vec3::new(0.0, 1.0, 1.0)),
                    WallOfStonePreview,
                    OnGameplayScreen,
                ))
                .id();

            caster.preview_entity = Some(preview_entity);
        }
    }

    // Update preview during casting
    if matches!(*casting_state, CastingState::Casting { .. }) {
        if let Some(anchor) = caster.anchor
            && let Some(preview_entity) = caster.preview_entity
            && let Ok(mut preview_transform) = preview_query.get_mut(preview_entity)
            && let Some(pos) = clamped_pos
        {
            let diff = Vec3::new(pos.x - anchor.x, 0.0, pos.z - anchor.z);
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

    // Despawn preview on completion or cancel
    if cast_result.completed || cast_result.despawn_preview {
        if let Some(preview_entity) = caster.preview_entity {
            commands.entity(preview_entity).despawn();
        }
        caster.preview_entity = None;
    }

    if cast_result.completed {
        mouse_state.left_consumed = true;

        if skip_spawn {
            if let (Some(conn), Some(anchor), Some(end)) =
                (connection.as_mut(), cast_result.anchor, cast_result.end_pos)
            {
                conn.outgoing_messages.push(NetworkMessage::SpellResult(
                    SpellAction::DragSpellCast {
                        spell: Spell::WallOfStone,
                        anchor: [anchor.x, anchor.y, anchor.z],
                        end_pos: [end.x, end.y, end.z],
                        empowerment: primed_spell.empowerment,
                    },
                ));
            }
        }
    }
}

/// Core Wall of Stone casting logic.
///
/// When `skip_spawn` is true, the casting pipeline runs normally but
/// wall spawning and obstacle events are skipped.
#[allow(clippy::too_many_arguments)]
fn wall_of_stone_casting_logic(
    input: &WizardInput,
    clamped_pos: Option<Vec3>,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster: &mut WallOfStoneCaster,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    skip_spawn: bool,
) -> CastResult {
    let mut result = CastResult {
        completed: false,
        despawn_preview: false,
        anchor: None,
        end_pos: None,
    };

    let Some(clamped_pos) = clamped_pos else {
        return result;
    };

    // Handle release — place wall or cancel
    if input.just_released {
        if let Some(anchor) = caster.anchor {
            let diff = Vec3::new(clamped_pos.x - anchor.x, 0.0, clamped_pos.z - anchor.z);
            let length = diff.length();

            if length >= MIN_WALL_LENGTH && mana.can_afford(MANA_COST) {
                let clamped_length = length.min(MAX_WALL_LENGTH);
                let forward = diff.normalize();
                let right = Vec3::new(-forward.z, 0.0, forward.x);
                let center = anchor + forward * (clamped_length / 2.0);

                mana.consume(MANA_COST);

                if !skip_spawn {
                    // Spawn the actual wall
                    let wall_mesh = Cuboid::new(clamped_length, WALL_HEIGHT, WALL_WIDTH);
                    let rotation = Quat::from_rotation_arc(Vec3::X, forward);

                    // Apply empowerment scaling
                    let scale = primed_spell.empowerment;
                    let wall_width = WALL_WIDTH * scale;
                    let wall_height = WALL_HEIGHT * scale;
                    let wall_duration = WALL_DURATION * scale;

                    commands.spawn((
                        Mesh3d(meshes.add(wall_mesh)),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: WALL_COLOR,
                            ..default()
                        })),
                        Transform::from_xyz(center.x, wall_height / 2.0, center.z)
                            .with_rotation(rotation),
                        WallOfStone {
                            center,
                            half_length: clamped_length / 2.0,
                            half_width: wall_width / 2.0,
                            forward,
                            right,
                            height: wall_height,
                            time_alive: 0.0,
                            duration: wall_duration,
                            sinking: false,
                            empowerment: primed_spell.empowerment,
                        },
                        NetworkedSpellEffect { kind: SpellEffectKind::WallOfStone },
                        OnGameplayScreen,
                    ));

                    // Notify pathfinding system about the new obstacle
                    let unbuffered_min_x =
                        center.x - forward.x * (clamped_length / 2.0) - right.x * (wall_width / 2.0);
                    let unbuffered_max_x =
                        center.x + forward.x * (clamped_length / 2.0) + right.x * (wall_width / 2.0);
                    let unbuffered_min_z =
                        center.z - forward.z * (clamped_length / 2.0) - right.z * (wall_width / 2.0);
                    let unbuffered_max_z =
                        center.z + forward.z * (clamped_length / 2.0) + right.z * (wall_width / 2.0);

                    let min_x = unbuffered_min_x.min(unbuffered_max_x) - OBSTACLE_BUFFER;
                    let max_x = unbuffered_min_x.max(unbuffered_max_x) + OBSTACLE_BUFFER;
                    let min_z = unbuffered_min_z.min(unbuffered_max_z) - OBSTACLE_BUFFER;
                    let max_z = unbuffered_min_z.max(unbuffered_max_z) + OBSTACLE_BUFFER;

                    obstacle_events.write(ObstacleChanged {
                        bounds: Rect::new(
                            min_x.min(max_x),
                            min_z.min(max_z),
                            (max_x - min_x).abs(),
                            (max_z - min_z).abs(),
                        ),
                        obstacle_type: ObstacleType::Blocked,
                    });
                }

                result.completed = true;
                result.anchor = Some(anchor);
                result.end_pos = Some(clamped_pos);
            } else {
                // Too short or can't afford — signal preview despawn
                result.despawn_preview = true;
            }

            caster.anchor = None;
            casting_state.cancel();
        }
        return result;
    }

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(MANA_COST) {
                caster.anchor = Some(clamped_pos);
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            // Preview update is handled by the local wrapper only
        }
        _ => {}
    }

    result
}

/// Handles right-click cancellation of wall placement.
pub fn handle_wall_of_stone_cancel(
    mut mouse_right_pressed: MessageReader<crate::game::input::messages::MouseRightPressed>,
    mut commands: Commands,
    mut wizard_query: Query<&mut CastingState, With<LocalWizard>>,
    mut caster_query: Query<&mut WallOfStoneCaster, With<LocalWizard>>,
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
pub fn cleanup_expired_walls(
    mut commands: Commands,
    walls: Query<(Entity, &WallOfStone)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, wall) in &walls {
        if wall.time_alive >= wall.duration {
            commands.entity(entity).despawn();

            // Notify pathfinding system that the obstacle is removed
            let unbuffered_min_x =
                wall.center.x - wall.forward.x * wall.half_length - wall.right.x * wall.half_width;
            let unbuffered_max_x =
                wall.center.x + wall.forward.x * wall.half_length + wall.right.x * wall.half_width;
            let unbuffered_min_z =
                wall.center.z - wall.forward.z * wall.half_length - wall.right.z * wall.half_width;
            let unbuffered_max_z =
                wall.center.z + wall.forward.z * wall.half_length + wall.right.z * wall.half_width;

            let min_x = unbuffered_min_x.min(unbuffered_max_x) - OBSTACLE_BUFFER;
            let max_x = unbuffered_min_x.max(unbuffered_max_x) + OBSTACLE_BUFFER;
            let min_z = unbuffered_min_z.min(unbuffered_max_z) - OBSTACLE_BUFFER;
            let max_z = unbuffered_min_z.max(unbuffered_max_z) + OBSTACLE_BUFFER;

            obstacle_events.write(ObstacleChanged {
                bounds: Rect::new(
                    min_x.min(max_x),
                    min_z.min(max_z),
                    (max_x - min_x).abs(),
                    (max_z - min_z).abs(),
                ),
                obstacle_type: ObstacleType::Removed,
            });
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
