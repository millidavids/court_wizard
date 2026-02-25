use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::components::{WallOfStone, WallOfStoneCaster, WallOfStonePreview};
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleType};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;

/// Result from spell casting logic, used to communicate state back to the wrapper.
struct CastResult {
    /// Whether the spell completed (wall was placed).
    completed: bool,
    /// Whether preview should be despawned (release with too-short drag or no mana).
    despawn_preview: bool,
    /// Obstacle bounds for network sync (set when completed=true).
    obstacle_bounds: Option<[f32; 4]>,
}

/// Local wizard Wall of Stone casting — reads mouse input, manages preview.
#[allow(clippy::too_many_arguments)]
pub fn handle_wall_of_stone_casting(
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visual_assets: Res<SpellVisualAssets>,
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
    mut connection: Option<ResMut<crate::networking::resources::NetworkConnection>>,
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

    let clamped_pos = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range(pos, wizard_transform.translation, wizard.spell_range));

    let cast_result = wall_of_stone_casting_logic(
        &input,
        clamped_pos,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut caster,
        &mut commands,
        &visual_assets,
        &mut obstacle_events,
    );

    // Send wall placement over network so the other client updates pathfinding
    if cast_result.completed {
        if let Some(bounds) = cast_result.obstacle_bounds {
            if let Some(ref mut conn) = connection {
                conn.outgoing_messages.push(
                    crate::networking::protocol::NetworkMessage::WallPlaced {
                        bounds,
                        placed: true,
                    },
                );
            }
        }
    }

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
                    Mesh3d(visual_assets.unit_cuboid.clone()),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: WALL_PREVIEW_COLOR,
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        cull_mode: None,
                        ..default()
                    })),
                    Transform::from_xyz(pos.x, WALL_HEIGHT / 2.0, pos.z).with_scale(Vec3::new(
                        0.0,
                        WALL_HEIGHT,
                        WALL_WIDTH,
                    )),
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

                preview_transform.translation = Vec3::new(center.x, WALL_HEIGHT / 2.0, center.z);
                preview_transform.rotation = rotation;
                preview_transform.scale = Vec3::new(length, WALL_HEIGHT, WALL_WIDTH);
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
    }
}

/// Core Wall of Stone casting logic.
#[allow(clippy::too_many_arguments)]
fn wall_of_stone_casting_logic(
    input: &WizardInput,
    clamped_pos: Option<Vec3>,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster: &mut WallOfStoneCaster,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) -> CastResult {
    let mut result = CastResult {
        completed: false,
        despawn_preview: false,
        obstacle_bounds: None,
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

                // Spawn the actual wall
                let rotation = Quat::from_rotation_arc(Vec3::X, forward);

                // Apply empowerment scaling
                let scale = primed_spell.empowerment;
                let wall_width = WALL_WIDTH * scale;
                let wall_height = WALL_HEIGHT * scale;
                let wall_duration = WALL_DURATION * scale;

                commands.spawn((
                    Mesh3d(assets.unit_cuboid.clone()),
                    MeshMaterial3d(assets.wall_of_stone.clone()),
                    Transform::from_xyz(center.x, wall_height / 2.0, center.z)
                        .with_rotation(rotation)
                        .with_scale(Vec3::new(clamped_length, wall_height, wall_width)),
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
                    NetworkedSpellEffect {
                        kind: SpellEffectKind::WallOfStone,
                    },
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

                let obs_bounds = [
                    min_x.min(max_x),
                    min_z.min(max_z),
                    (max_x - min_x).abs(),
                    (max_z - min_z).abs(),
                ];

                obstacle_events.write(ObstacleChanged {
                    bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
                    obstacle_type: ObstacleType::Blocked,
                });

                result.completed = true;
                result.obstacle_bounds = Some(obs_bounds);
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
    mut connection: Option<ResMut<crate::networking::resources::NetworkConnection>>,
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

            let obs_bounds = [
                min_x.min(max_x),
                min_z.min(max_z),
                (max_x - min_x).abs(),
                (max_z - min_z).abs(),
            ];

            obstacle_events.write(ObstacleChanged {
                bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
                obstacle_type: ObstacleType::Removed,
            });

            // Notify remote peer to update their pathfinding grid
            if let Some(ref mut conn) = connection {
                conn.outgoing_messages.push(
                    crate::networking::protocol::NetworkMessage::WallPlaced {
                        bounds: obs_bounds,
                        placed: false,
                    },
                );
            }
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
