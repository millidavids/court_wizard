use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, Mana, PrimedSpell, SpellCaster, Wizard};
use super::components::GuardianCircleIndicator;
use super::constants;
use super::styles::CIRCLE_COLOR;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::input::events::MouseLeftReleased;
use crate::game::units::components::TemporaryHitPoints;

/// Handles Guardian Circle casting with left-click.
///
/// Left-click starts cast. Must hold for full cast time.
/// After cast completes, applies temporary HP to all units in radius.
/// Only casts when Guardian Circle is the primed spell.
///
/// Note: Spell priming, input blocking, and mouse state checks are handled by run_if conditions.
#[allow(clippy::too_many_arguments)]
pub fn handle_guardian_circle_casting(
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
        ),
        With<Wizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    caster_query: Query<&SpellCaster, With<Wizard>>,
    mut indicator_query: Query<&mut GuardianCircleIndicator>,
    mut targets_query: Query<(Entity, &Transform), Without<Wizard>>,
) {
    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };

    // Check for release event - this is spell-specific logic
    if mouse_left_released.read().next().is_some() {
        // Cancel cast on release
        if let Ok(caster) = caster_query.single() {
            // Despawn circle indicator if it exists
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).despawn();
            }
            // Remove caster marker
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
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
    // Scale radius by empowerment
    let scale = primed_spell.empowerment;
    let circle_radius = constants::CIRCLE_RADIUS * scale;
    let max_center_distance = (max_ground_radius - circle_radius).max(0.0);

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
            // Only start a new cast if we don't have a caster marker and have enough mana
            if caster_query.get(wizard_entity).is_err() && mana.can_afford(constants::MANA_COST) {
                // Start casting - spawn circle indicator
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    cursor_world_pos,
                    primed_spell.empowerment,
                );

                // Mark wizard as casting Guardian Circle
                commands
                    .entity(wizard_entity)
                    .insert(SpellCaster::with_indicator(circle_entity));

                // Start the cast
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            // Currently casting - advance cast time
            casting_state.advance(time.delta_secs());

            // Update circle position to follow cursor
            if let Ok(caster) = caster_query.single()
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = cursor_world_pos;
            }

            // Check if cast is complete
            if casting_state.is_complete(primed_spell.cast_time) {
                // Cast complete - apply buff to units in radius
                if mana.consume(constants::MANA_COST) {
                    // Get final circle position and apply buff
                    if let Ok(caster) = caster_query.single() {
                        if let Some(indicator_entity) = caster.indicator_entity {
                            if let Ok(indicator) = indicator_query.get(indicator_entity) {
                                // Scale radius by empowerment
                                let scale = indicator.empowerment;
                                let radius = constants::CIRCLE_RADIUS * scale;

                                apply_guardian_circle_buff(
                                    &mut commands,
                                    indicator.position,
                                    radius,
                                    constants::TEMP_HP_AMOUNT,
                                    constants::TEMP_HP_DURATION,
                                    indicator.empowerment,
                                    &mut targets_query,
                                );
                            }

                            // Despawn circle indicator
                            commands.entity(indicator_entity).despawn();
                        }
                    }

                    // Remove caster marker immediately (don't keep it blocking future casts)
                    commands.entity(wizard_entity).remove::<SpellCaster>();

                    // Return to resting state
                    casting_state.cancel();
                    // Consume mouse to require release before next cast
                    mouse_state.left_consumed = true;
                } else {
                    // Out of mana - cancel cast
                    if let Ok(caster) = caster_query.single() {
                        if let Some(indicator_entity) = caster.indicator_entity {
                            commands.entity(indicator_entity).despawn();
                        }
                        commands.entity(wizard_entity).remove::<SpellCaster>();
                    }
                    casting_state.cancel();
                }
            }
        }
        CastingState::Channeling { .. } => {
            // Guardian Circle doesn't use channeling, cancel if we somehow get here
            if let Ok(caster) = caster_query.single() {
                if let Some(indicator_entity) = caster.indicator_entity {
                    commands.entity(indicator_entity).despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
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
/// Scales temp HP amount and duration by 1.25x when empowered.
fn apply_guardian_circle_buff(
    commands: &mut Commands,
    circle_pos: Vec3,
    radius: f32,
    temp_hp_amount: f32,
    duration: f32,
    empowerment: f32,
    targets: &mut Query<(Entity, &Transform), Without<Wizard>>,
) {
    // Scale values by empowerment
    let scale = empowerment;
    let scaled_temp_hp = temp_hp_amount * scale;
    let scaled_duration = duration * scale;

    for (entity, transform) in targets.iter() {
        let distance = transform.translation.distance(circle_pos);

        if distance <= radius {
            // Unit is in range - add or update TemporaryHitPoints
            commands
                .entity(entity)
                .insert(TemporaryHitPoints::new(scaled_temp_hp, scaled_duration));
        }
    }
}

/// Helper function to spawn the visual circle indicator.
///
/// Creates a translucent cyan circle mesh at the target position.
/// Scales radius by 1.25x when empowered.
fn spawn_circle_indicator(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    empowerment: f32,
) -> Entity {
    // Scale radius by empowerment
    let scale = empowerment;
    let radius = constants::CIRCLE_RADIUS * scale;

    let circle_mesh = meshes.add(Circle::new(radius));
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
            GuardianCircleIndicator::new(position, empowerment),
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
