//! Systems for the Teleport spell.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

use super::super::super::components::{CastingState, GuestWizard, Mana, PrimedSpell, Spell, LocalWizard, Wizard};
use super::components::{TeleportCaster, TeleportDestinationCircle, TeleportSourceCircle};
use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::BATTLEFIELD_SIZE;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::{MouseLeftReleased, MouseRightPressed};
use crate::game::multiplayer::spell_commands::{GuestCursorPosition, GuestInputState};
use crate::game::units::components::Teleportable;

/// Handles right-click to cancel/reset the teleport spell.
///
/// This system runs independently of the main casting system to ensure
/// right-click always cancels, even when other conditions would block casting.
pub fn handle_teleport_cancel(
    mut mouse_right_pressed: MessageReader<MouseRightPressed>,
    mut commands: Commands,
    mut wizard_query: Query<(&mut CastingState, Entity), With<LocalWizard>>,
    mut caster_query: Query<&mut TeleportCaster, With<LocalWizard>>,
    mut mouse_state: ResMut<MouseButtonState>,
) {
    // Only process if right-click occurred
    if mouse_right_pressed.read().next().is_none() {
        return;
    }

    // Get wizard and caster
    let Ok((mut casting_state, wizard_entity)) = wizard_query.single_mut() else {
        return;
    };

    let mut caster = if let Ok(c) = caster_query.single_mut() {
        c
    } else {
        commands.entity(wizard_entity).insert(TeleportCaster::new());
        return;
    };

    // Despawn any active circles
    if let Some(dest_entity) = caster.destination_circle {
        commands.entity(dest_entity).despawn();
    }
    if let Some(source_entity) = caster.source_circle {
        commands.entity(source_entity).despawn();
    }

    // Reset all state
    caster.destination_circle = None;
    caster.destination_position = None;
    caster.source_circle = None;
    casting_state.cancel();
    mouse_state.left_consumed = true; // Prevent immediate restart if left button still held
}

/// Handles Teleport spell casting with two phases.
///
/// Phase 1: Place destination circle (1 second cast)
/// Phase 2: Place source circle and teleport units (2 second cast)
///
/// Note: Spell priming, input blocking, and mouse state checks are handled by run_if conditions.
#[allow(clippy::too_many_arguments)]
pub fn handle_teleport_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
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
            Option<&GuestWizard>,
        ),
        (
            Or<(With<LocalWizard>, With<GuestWizard>)>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
        ),
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut caster_query: Query<&mut TeleportCaster>,
    mut destination_query: Query<
        (&mut Transform, &mut TeleportDestinationCircle),
        (
            With<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
        ),
    >,
    mut source_query: Query<
        (&mut Transform, &mut TeleportSourceCircle),
        (
            With<TeleportSourceCircle>,
            Without<TeleportDestinationCircle>,
        ),
    >,
    units_query: Query<
        (Entity, &Transform),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
        ),
    >,
    guest_cursor: Option<Res<GuestCursorPosition>>,
    guest_input: Option<Res<GuestInputState>>,
) {
    // Read local input once before the loop
    let local_released = mouse_left_released.read().next().is_some();

    for (wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell, is_guest) in wizard_query.iter_mut() {
        if primed_spell.spell != Spell::Teleport { continue; }

        let is_guest = is_guest.is_some();

        // Get or create caster component
        let mut caster = if let Ok(c) = caster_query.get_mut(wizard_entity) {
            c
        } else {
            commands.entity(wizard_entity).insert(TeleportCaster::new());
            continue; // Wait for next frame to query it
        };

        // Safety check - if consumed is somehow true, don't do anything
        // This shouldn't happen due to run_if conditions, but prevents edge cases
        if !is_guest && mouse_state.left_consumed {
            continue;
        }

        let released = if is_guest {
            guest_input.as_ref().is_some_and(|i| i.just_released)
        } else {
            local_released
        };

        // Handle release during first cast - finalize destination position
        if released
            && !caster.has_destination()
            && matches!(*casting_state, CastingState::Casting { .. })
        {
            let cursor_pos = if is_guest {
                guest_cursor.as_ref().and_then(|c| c.position)
            } else {
                get_cursor_world_position(&camera_query, &window_query)
            };
            if let Some(cursor_world_pos) = cursor_pos {
                let wizard_pos = wizard_transform.translation;
                let clamped_pos =
                    clamp_to_spell_range(cursor_world_pos, wizard_pos, wizard.spell_range);

                caster.destination_position = Some(clamped_pos);
                casting_state.cancel(); // Return to resting for phase 2
                if !is_guest {
                    mouse_state.left_consumed = true; // Require new click for second cast
                }
            }
            continue;
        }

        // Handle release during second cast - completes teleport early
        if released && caster.has_destination() && caster.source_circle.is_some() {
            if let CastingState::Casting { elapsed } = *casting_state {
                // Get source circle position
                if let Some(source_entity) = caster.source_circle
                    && let Ok((transform, source_circle)) = source_query.get(source_entity)
                {
                    let source_pos = transform.translation;

                    // Calculate current circle radius based on growth
                    let growth = (elapsed / SECOND_CAST_TIME).min(1.0);
                    let scale = source_circle.empowerment;
                    let current_radius = CIRCLE_RADIUS * scale * growth;

                    // Check mana and execute teleport
                    if mana.can_afford(MANA_COST) {
                        mana.consume(MANA_COST);

                        if let Some(dest_pos) = caster.destination_position {
                            teleport_units_with_radius(
                                source_pos,
                                dest_pos,
                                current_radius,
                                &units_query,
                                &mut commands,
                            );
                        }

                        // Cleanup
                        if !is_guest {
                            if let Some(dest_entity) = caster.destination_circle {
                                commands.entity(dest_entity).despawn();
                            }
                            commands.entity(source_entity).despawn();
                        }

                        caster.destination_circle = None;
                        caster.destination_position = None;
                        caster.source_circle = None;

                        casting_state.cancel();
                        if !is_guest {
                            mouse_state.left_consumed = true;
                        }
                    }
                }
            }
            continue;
        }

        // Get cursor world position
        let cursor_pos = if is_guest {
            guest_cursor.as_ref().and_then(|c| c.position)
        } else {
            get_cursor_world_position(&camera_query, &window_query)
        };
        let Some(cursor_world_pos) = cursor_pos else {
            continue;
        };

        // Clamp to spell range
        let wizard_pos = wizard_transform.translation;
        let clamped_pos = clamp_to_spell_range(cursor_world_pos, wizard_pos, wizard.spell_range);

        // State machine based on whether destination exists
        if !caster.has_destination() {
            // PHASE 1: Placing destination circle
            handle_first_cast(
                &mut casting_state,
                &mut caster,
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut destination_query,
                clamped_pos,
                primed_spell,
                is_guest,
                &guest_input,
            );
        } else {
            // PHASE 2: Placing source circle and teleporting
            handle_second_cast(
                &time,
                &mut casting_state,
                &mut mouse_state,
                &mut mana,
                &mut caster,
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut source_query,
                clamped_pos,
                primed_spell,
                &units_query,
                is_guest,
                &guest_input,
            );
        }
    }
}

/// Handles the first cast phase (destination placement) - shows crosshair while mouse is held.
#[allow(clippy::too_many_arguments)]
fn handle_first_cast(
    casting_state: &mut CastingState,
    caster: &mut TeleportCaster,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    destination_query: &mut Query<
        (&mut Transform, &mut TeleportDestinationCircle),
        (
            With<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
        ),
    >,
    position: Vec3,
    primed_spell: &PrimedSpell,
    is_guest: bool,
    guest_input: &Option<Res<GuestInputState>>,
) {
    match *casting_state {
        CastingState::Resting => {
            let has_input = if is_guest {
                guest_input.as_ref().is_some_and(|i| i.just_pressed || i.pressed)
            } else {
                true // Run conditions already ensure mouse is held for local wizard
            };
            if !has_input {
                return;
            }

            if !is_guest {
                // Start showing crosshair on mouse down (local wizard only)
                let radius = primed_spell.scale(CROSSHAIR_RADIUS);
                let crosshair_mesh = meshes.add(Circle::new(radius));
                let crosshair_material = materials.add(StandardMaterial {
                    base_color: DESTINATION_COLOR,
                    unlit: true,
                    ..default()
                });

                let crosshair_entity = commands
                    .spawn((
                        Mesh3d(crosshair_mesh),
                        MeshMaterial3d(crosshair_material),
                        Transform::from_xyz(position.x, 1.0, position.z)
                            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                        TeleportDestinationCircle::new(primed_spell.empowerment),
                        OnGameplayScreen,
                    ))
                    .id();

                caster.destination_circle = Some(crosshair_entity);
            }
            casting_state.start_cast(); // Enter casting state to track mouse movement
        }
        CastingState::Casting { .. } => {
            // Update crosshair position to follow mouse while button is held (local only)
            if !is_guest {
                if let Some(circle_entity) = caster.destination_circle
                    && let Ok((mut transform, _)) = destination_query.get_mut(circle_entity)
                {
                    transform.translation.x = position.x;
                    transform.translation.z = position.z;
                }
            }
        }
        CastingState::Channeling { .. } => {
            // Not used for teleport
        }
    }
}

/// Handles the second cast phase (source placement and teleportation).
#[allow(clippy::too_many_arguments)]
fn handle_second_cast(
    time: &Res<Time>,
    casting_state: &mut CastingState,
    mouse_state: &mut ResMut<MouseButtonState>,
    mana: &mut Mana,
    caster: &mut TeleportCaster,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    source_query: &mut Query<
        (&mut Transform, &mut TeleportSourceCircle),
        (
            With<TeleportSourceCircle>,
            Without<TeleportDestinationCircle>,
        ),
    >,
    position: Vec3,
    primed_spell: &PrimedSpell,
    units_query: &Query<
        (Entity, &Transform),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
        ),
    >,
    is_guest: bool,
    guest_input: &Option<Res<GuestInputState>>,
) {
    match *casting_state {
        CastingState::Resting => {
            // Check mana before starting second cast
            if !mana.can_afford(MANA_COST) {
                return;
            }

            let has_input = if is_guest {
                guest_input.as_ref().is_some_and(|i| i.just_pressed || i.pressed)
            } else {
                true // Run conditions already ensure mouse is held for local wizard
            };
            if !has_input {
                return;
            }

            // Start casting second phase
            casting_state.start_cast();

            if !is_guest {
                // Spawn source circle (local wizard only)
                let radius = primed_spell.scale(CIRCLE_RADIUS);
                let circle_mesh = meshes.add(Circle::new(radius));
                let circle_material = materials.add(StandardMaterial {
                    base_color: SOURCE_COLOR,
                    unlit: true,
                    ..default()
                });

                let circle_entity = commands
                    .spawn((
                        Mesh3d(circle_mesh),
                        MeshMaterial3d(circle_material),
                        Transform::from_xyz(position.x, 1.0, position.z)
                            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                            .with_scale(Vec3::ZERO), // Start at zero size
                        TeleportSourceCircle::new(position, primed_spell.empowerment),
                        OnGameplayScreen,
                    ))
                    .id();

                caster.source_circle = Some(circle_entity);
            }
        }
        CastingState::Casting { ref mut elapsed } => {
            // Advance cast
            *elapsed += time.delta_secs();

            // Update circle position during cast (local wizard only)
            if !is_guest {
                if let Some(circle_entity) = caster.source_circle
                    && let Ok((mut transform, mut indicator)) = source_query.get_mut(circle_entity)
                {
                    transform.translation.x = position.x;
                    transform.translation.z = position.z;

                    // Grow circle from 0 to full radius
                    let growth = (*elapsed / SECOND_CAST_TIME).min(1.0);
                    transform.scale = Vec3::splat(growth);

                    indicator.position = position;
                    indicator.time_alive += time.delta_secs();
                }
            }

            // Check if cast complete
            if *elapsed >= SECOND_CAST_TIME {
                // Consume mana
                mana.consume(MANA_COST);

                // Execute teleportation with scaled radius
                let radius = primed_spell.scale(CIRCLE_RADIUS);
                if let Some(dest_pos) = caster.destination_position {
                    teleport_units(position, dest_pos, radius, units_query, commands);
                }

                if !is_guest {
                    // Despawn both circles (local wizard only)
                    if let Some(dest_entity) = caster.destination_circle {
                        commands.entity(dest_entity).despawn();
                    }
                    if let Some(source_entity) = caster.source_circle {
                        commands.entity(source_entity).despawn();
                    }
                }

                // Reset caster state completely
                caster.destination_circle = None;
                caster.destination_position = None;
                caster.source_circle = None;

                casting_state.cancel(); // Return to resting immediately
                if !is_guest {
                    mouse_state.left_consumed = true; // Prevent immediate restart while mouse held
                }
            }
        }
        _ => {}
    }
}

/// Teleports all units within the source circle to random positions within the destination circle.
fn teleport_units(
    source_center: Vec3,
    dest_center: Vec3,
    radius: f32,
    units_query: &Query<
        (Entity, &Transform),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
        ),
    >,
    commands: &mut Commands,
) {
    teleport_units_with_radius(source_center, dest_center, radius, units_query, commands);
}

/// Teleports all units within a specified radius of the source center to random positions
/// within the same radius of the destination center.
fn teleport_units_with_radius(
    source_center: Vec3,
    dest_center: Vec3,
    radius: f32,
    units_query: &Query<
        (Entity, &Transform),
        (
            With<Teleportable>,
            Without<TeleportDestinationCircle>,
            Without<TeleportSourceCircle>,
        ),
    >,
    commands: &mut Commands,
) {
    let mut rng = rand::thread_rng();

    for (entity, transform) in units_query.iter() {
        // Check if unit is within source circle (XZ distance only)
        let diff_x = transform.translation.x - source_center.x;
        let diff_z = transform.translation.z - source_center.z;
        let distance = (diff_x * diff_x + diff_z * diff_z).sqrt();

        if distance <= radius {
            // Generate random position within destination circle
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let random_radius = rng.gen_range(0.0..radius);

            let offset_x = angle.cos() * random_radius;
            let offset_z = angle.sin() * random_radius;

            let new_x = dest_center.x + offset_x;
            let new_z = dest_center.z + offset_z;

            // Clamp to battlefield bounds
            let clamped_x = new_x.clamp(-BATTLEFIELD_SIZE / 2.0, BATTLEFIELD_SIZE / 2.0);
            let clamped_z = new_z.clamp(-BATTLEFIELD_SIZE / 2.0, BATTLEFIELD_SIZE / 2.0);

            // Keep original Y position and rotation
            let new_position = Vec3::new(clamped_x, transform.translation.y, clamped_z);

            let mut new_transform = *transform;
            new_transform.translation = new_position;

            commands.entity(entity).insert(new_transform);
        }
    }
}

/// Updates pulse animations for both destination and source circles.
pub fn update_circle_animations(
    time: Res<Time>,
    mut destination_query: Query<
        (&mut Transform, &mut TeleportDestinationCircle),
        Without<TeleportSourceCircle>,
    >,
    mut source_query: Query<(&mut Transform, &mut TeleportSourceCircle)>,
) {
    // Update destination circles
    for (mut transform, mut indicator) in &mut destination_query {
        indicator.time_alive += time.delta_secs();

        // Only apply pulse animation after growth is mostly complete
        if transform.scale.x >= PULSE_THRESHOLD {
            let pulse = indicator.pulse_scale();
            transform.scale = Vec3::splat(pulse);
        }
    }

    // Update source circles
    for (mut transform, mut indicator) in &mut source_query {
        indicator.time_alive += time.delta_secs();

        // Only apply pulse animation after growth is mostly complete
        if transform.scale.x >= PULSE_THRESHOLD {
            let pulse = indicator.pulse_scale();
            transform.scale = Vec3::splat(pulse);
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
