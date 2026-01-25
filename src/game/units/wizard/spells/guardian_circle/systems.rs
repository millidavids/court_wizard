use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::components::{GuardianCircleCaster, GuardianCircleIndicator};
use super::constants;
use super::styles::CIRCLE_COLOR;
use crate::game::components::OnGameplayScreen;
use crate::game::input::events::{BlockSpellInput, MouseLeftHeld, MouseLeftReleased};
use crate::game::units::components::TemporaryHitPoints;
use crate::game::units::wizard::components::{CastingState, Mana, PrimedSpell, Spell, Wizard};

/// Handles Guardian Circle casting with left-click.
///
/// Left-click starts cast. Must hold for full cast time.
/// After cast completes, applies temporary HP to all units in radius.
/// Only casts when Guardian Circle is the primed spell.
#[allow(clippy::too_many_arguments)]
pub fn handle_guardian_circle_casting(
    time: Res<Time>,
    mut block_spell_input: MessageReader<BlockSpellInput>,
    mut mouse_left_held: MessageReader<MouseLeftHeld>,
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
        ),
        With<Wizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut caster_query: Query<&mut GuardianCircleCaster, With<Wizard>>,
    mut indicator_query: Query<&mut GuardianCircleIndicator>,
    mut targets_query: Query<(Entity, &Transform), Without<Wizard>>,
) {
    // Don't cast if spell input is blocked (UI button was clicked)
    if block_spell_input.read().next().is_some() {
        return;
    }

    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };

    // Only respond to left-click if Guardian Circle is primed
    if primed_spell.spell != Spell::GuardianCircle {
        return;
    }

    // Check for release event
    if mouse_left_released.read().next().is_some() {
        // Cancel cast on release
        if let Ok(caster) = caster_query.single() {
            // Despawn circle indicator if it exists
            if let Some(circle_entity) = caster.circle_entity {
                commands.entity(circle_entity).despawn();
            }
            // Remove caster marker
            commands
                .entity(wizard_entity)
                .remove::<GuardianCircleCaster>();
        }
        casting_state.cancel();
        return;
    }

    // Check for hold event
    if mouse_left_held.read().next().is_none() {
        return;
    }

    // Get cursor world position and clamp to wizard's spell range
    let Some(mut cursor_world_pos) = get_cursor_world_position(&camera_query, &window_query) else {
        return;
    };

    // Clamp cursor position to be within wizard's spell range
    // Use the same 3D distance calculation as the spell range indicator
    let wizard_pos = wizard_transform.translation;
    let wizard_height = wizard_pos.y;

    // Calculate the actual ground circle radius using Pythagorean theorem
    // spell_range² = circle_radius² + wizard_height²
    // Therefore: circle_radius = √(spell_range² - wizard_height²)
    let max_ground_radius = if wizard_height < wizard.spell_range {
        (wizard.spell_range * wizard.spell_range - wizard_height * wizard_height).sqrt()
    } else {
        0.0
    };

    // Account for the Guardian Circle's radius so the entire circle stays within range
    let max_center_distance = (max_ground_radius - constants::CIRCLE_RADIUS).max(0.0);

    // Calculate XZ plane distance from wizard to cursor
    let direction = cursor_world_pos - wizard_pos;
    let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();

    if distance > max_center_distance && distance > 0.001 {
        // Clamp to ensure the entire circle stays within spell range
        let normalized_direction = direction / distance;
        cursor_world_pos = wizard_pos + normalized_direction * max_center_distance;
    }

    // Mouse is held - handle casting based on state
    match *casting_state {
        CastingState::Resting => {
            // Only start a new cast if we don't have a caster marker
            // (the marker persists after cast completion until mouse release)
            if caster_query.single().is_err() {
                // Start casting - spawn circle indicator
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    cursor_world_pos,
                );

                // Mark wizard as casting Guardian Circle
                commands.entity(wizard_entity).insert(GuardianCircleCaster {
                    circle_entity: Some(circle_entity),
                });

                // Start the cast
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            // Currently casting - advance cast time
            casting_state.advance(time.delta_secs());

            // Update circle position to follow cursor
            if let Ok(caster) = caster_query.single()
                && let Some(circle_entity) = caster.circle_entity
                && let Ok(mut indicator) = indicator_query.get_mut(circle_entity)
            {
                indicator.position = cursor_world_pos;
            }

            // Check if cast is complete
            if casting_state.is_complete(primed_spell.cast_time) {
                // Cast complete - apply buff to units in radius
                if mana.consume(constants::MANA_COST) {
                    // Get final circle position and apply buff
                    if let Ok(mut caster) = caster_query.single_mut() {
                        if let Some(circle_entity) = caster.circle_entity {
                            if let Ok(indicator) = indicator_query.get(circle_entity) {
                                apply_guardian_circle_buff(
                                    &mut commands,
                                    indicator.position,
                                    constants::CIRCLE_RADIUS,
                                    constants::TEMP_HP_AMOUNT,
                                    constants::TEMP_HP_DURATION,
                                    &mut targets_query,
                                );
                            }

                            // Despawn circle indicator
                            commands.entity(circle_entity).despawn();
                        }

                        // Clear circle entity reference but keep marker to prevent immediate recast
                        caster.circle_entity = None;
                    }

                    // Return to resting state
                    casting_state.cancel();
                } else {
                    // Out of mana - cancel cast
                    if let Ok(caster) = caster_query.single() {
                        if let Some(circle_entity) = caster.circle_entity {
                            commands.entity(circle_entity).despawn();
                        }
                        commands
                            .entity(wizard_entity)
                            .remove::<GuardianCircleCaster>();
                    }
                    casting_state.cancel();
                }
            }
        }
        CastingState::Channeling { .. } => {
            // Guardian Circle doesn't use channeling, cancel if we somehow get here
            if let Ok(caster) = caster_query.single() {
                if let Some(circle_entity) = caster.circle_entity {
                    commands.entity(circle_entity).despawn();
                }
                commands
                    .entity(wizard_entity)
                    .remove::<GuardianCircleCaster>();
            }
            casting_state.cancel();
        }
    }
}

/// Updates circle indicator visuals during casting.
///
/// Applies pulse animation and updates position/scale.
pub fn update_circle_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut GuardianCircleIndicator, &mut Transform)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        // Update time alive for pulse animation
        indicator.time_alive += time.delta_secs();

        // Apply pulse scale (preserve rotation by only scaling)
        let pulse = indicator.pulse_scale();
        transform.scale = Vec3::splat(pulse);

        // Update position (preserve rotation by only updating translation)
        transform.translation.x = indicator.position.x;
        transform.translation.y = constants::CIRCLE_Y_POSITION;
        transform.translation.z = indicator.position.z;
    }
}

/// Helper function to apply Guardian Circle buff to all units in radius.
///
/// Grants temporary HP to units. If a unit already has temp HP, takes the maximum.
fn apply_guardian_circle_buff(
    commands: &mut Commands,
    circle_pos: Vec3,
    radius: f32,
    temp_hp_amount: f32,
    duration: f32,
    targets: &mut Query<(Entity, &Transform), Without<Wizard>>,
) {
    for (entity, transform) in targets.iter() {
        let distance = transform.translation.distance(circle_pos);

        if distance <= radius {
            // Unit is in range - add or update TemporaryHitPoints
            commands
                .entity(entity)
                .insert(TemporaryHitPoints::new(temp_hp_amount, duration));
        }
    }
}

/// Helper function to spawn the visual circle indicator.
///
/// Creates a translucent cyan circle mesh at the target position.
fn spawn_circle_indicator(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) -> Entity {
    let circle_mesh = meshes.add(Circle::new(constants::CIRCLE_RADIUS));
    let circle_material = materials.add(StandardMaterial {
        base_color: CIRCLE_COLOR,
        unlit: true,
        ..default()
    });

    commands
        .spawn((
            Mesh3d(circle_mesh),
            MeshMaterial3d(circle_material),
            Transform::from_translation(Vec3::new(
                position.x,
                constants::CIRCLE_Y_POSITION,
                position.z,
            ))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            GuardianCircleIndicator::new(position),
            OnGameplayScreen,
        ))
        .id()
}

/// Helper function to get cursor world position at Y=0 plane.
///
/// Ray casts from camera through cursor to find intersection with ground plane.
fn get_cursor_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec3> {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return None;
    };
    let Ok(window) = window_query.single() else {
        return None;
    };

    let cursor_position = window.cursor_position()?;

    // Convert cursor position to world ray
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return None;
    };

    // Intersect ray with Y=0 plane
    // Ray equation: P = origin + t * direction
    // Plane equation: Y = 0
    // Solve for t: origin.y + t * direction.y = 0
    // t = -origin.y / direction.y

    if ray.direction.y.abs() < 0.0001 {
        return None; // Ray is parallel to plane
    }

    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        return None; // Intersection is behind camera
    }

    let intersection = ray.origin + ray.direction * t;
    Some(intersection)
}
