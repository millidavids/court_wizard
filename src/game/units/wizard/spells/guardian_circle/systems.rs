use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, GuestWizard, Mana, PrimedSpell, Spell, SpellCaster, LocalWizard, Wizard, WizardInput};
use super::components::GuardianCircleIndicator;
use super::constants;
use super::styles::CIRCLE_COLOR;
use crate::game::achievements::messages::GuardianCircleHitAttackerMessage;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::spell_commands::{GuestCursorPosition, GuestInputState};
use crate::game::units::components::{Team, TemporaryHitPoints};

/// Result from spell casting logic, used to communicate state back to the wrapper.
struct CastResult {
    /// Whether the spell completed (cast finished and effect spawned).
    completed: bool,
}

/// Local wizard Guardian Circle casting -- reads mouse input.
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
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut GuardianCircleIndicator>,
    mut targets_query: Query<(Entity, &Transform, &Team), Without<Wizard>>,
    mut attacker_hit_msg: MessageWriter<GuardianCircleHitAttackerMessage>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::GuardianCircle { return; }

    // Clamp cursor to spell range
    let clamped_cursor = clamp_cursor_to_range(input.cursor_pos, wizard_transform, wizard, primed_spell);

    // Handle release -- clean up indicator and SpellCaster
    if input.just_released {
        if let Ok(caster) = caster_query.get(wizard_entity) {
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).despawn();
            }
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return;
    }

    // Manage indicator based on casting state
    match *casting_state {
        CastingState::Resting => {
            if caster_query.get(wizard_entity).is_err()
                && mana.can_afford(constants::MANA_COST)
            {
                if let Some(pos) = clamped_cursor {
                    let circle_entity = spawn_circle_indicator(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        pos,
                        primed_spell.empowerment,
                    );
                    commands
                        .entity(wizard_entity)
                        .insert(SpellCaster::with_indicator(circle_entity));
                }
            }
        }
        CastingState::Casting { .. } => {
            if let Some(pos) = clamped_cursor {
                if let Ok(caster) = caster_query.get(wizard_entity)
                    && let Some(indicator_entity) = caster.indicator_entity
                    && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
                {
                    indicator.position = pos;
                }
            }
        }
        CastingState::Channeling { .. } => {
            if let Ok(caster) = caster_query.get(wizard_entity) {
                if let Some(indicator_entity) = caster.indicator_entity {
                    commands.entity(indicator_entity).despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
            }
        }
    }

    let cast_result = guardian_circle_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
    );

    if cast_result.completed {
        // Apply buff using indicator position
        if let Ok(caster) = caster_query.get(wizard_entity)
            && let Some(indicator_entity) = caster.indicator_entity
        {
            if let Ok(indicator) = indicator_query.get(indicator_entity) {
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
                    &mut attacker_hit_msg,
                );
            }
            commands.entity(indicator_entity).despawn();
        }
        commands.entity(wizard_entity).remove::<SpellCaster>();
        mouse_state.left_consumed = true;
    }
}

/// Guest wizard Guardian Circle casting -- reads network signals.
#[allow(clippy::too_many_arguments)]
pub fn handle_guardian_circle_casting_guest(
    time: Res<Time>,
    mut commands: Commands,
    mut wizard_query: Query<
        (
            Entity,
            &Transform,
            &Wizard,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
        ),
        With<GuestWizard>,
    >,
    caster_query: Query<&SpellCaster>,
    mut targets_query: Query<(Entity, &Transform, &Team), Without<Wizard>>,
    mut attacker_hit_msg: MessageWriter<GuardianCircleHitAttackerMessage>,
    guest_cursor: Res<GuestCursorPosition>,
    guest_input: Res<GuestInputState>,
) {
    let input = WizardInput {
        just_pressed: guest_input.just_pressed,
        pressed: guest_input.pressed,
        just_released: guest_input.just_released,
        cursor_pos: guest_cursor.position,
    };

    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::GuardianCircle { return; }

    // Clamp cursor to spell range
    let clamped_cursor = clamp_cursor_to_range(input.cursor_pos, wizard_transform, wizard, primed_spell);

    // Handle release
    if input.just_released {
        if caster_query.get(wizard_entity).is_ok() {
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return;
    }

    // Manage SpellCaster for guest (no indicator)
    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(constants::MANA_COST)
            {
                commands.entity(wizard_entity).insert(SpellCaster::new());
            }
        }
        CastingState::Channeling { .. } => {
            if caster_query.get(wizard_entity).is_ok() {
                commands.entity(wizard_entity).remove::<SpellCaster>();
            }
        }
        _ => {}
    }

    let cast_result = guardian_circle_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
    );

    if cast_result.completed {
        // Apply buff at cursor position for guest
        if let Some(pos) = clamped_cursor {
            let scale = primed_spell.empowerment;
            let radius = constants::CIRCLE_RADIUS * scale;

            apply_guardian_circle_buff(
                &mut commands,
                pos,
                radius,
                constants::TEMP_HP_AMOUNT,
                constants::TEMP_HP_DURATION,
                primed_spell.empowerment,
                &mut targets_query,
                &mut attacker_hit_msg,
            );
        }
        commands.entity(wizard_entity).remove::<SpellCaster>();
    }
}

/// Core Guardian Circle casting logic -- called by both local and guest systems.
///
/// Handles CastingState transitions and mana consumption.
/// Does NOT manage SpellCaster, indicators, or mouse_state -- those are the wrapper's job.
fn guardian_circle_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
) -> CastResult {
    let mut result = CastResult { completed: false };

    // Release is handled by the wrappers before calling this function
    if input.just_released {
        return result;
    }

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    result.completed = true;
                }
                casting_state.cancel();
            }
        }
        CastingState::Resting => {
            if input.just_pressed || input.pressed {
                casting_state.start_cast();
            }
        }
    }

    result
}

/// Clamps cursor position to spell range accounting for circle radius.
fn clamp_cursor_to_range(
    cursor_pos: Option<Vec3>,
    wizard_transform: &Transform,
    wizard: &Wizard,
    primed_spell: &PrimedSpell,
) -> Option<Vec3> {
    let mut pos = cursor_pos?;

    let wizard_pos = wizard_transform.translation;
    let wizard_height = wizard_pos.y;
    let max_ground_radius = if wizard_height < wizard.spell_range {
        (wizard.spell_range * wizard.spell_range - wizard_height * wizard_height).sqrt()
    } else {
        0.0
    };
    let scale = primed_spell.empowerment;
    let circle_radius = constants::CIRCLE_RADIUS * scale;
    let max_center_distance = (max_ground_radius - circle_radius).max(0.0);
    let direction = pos - wizard_pos;
    let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();
    if distance > max_center_distance && distance > 0.001 {
        let normalized_direction = direction / distance;
        pos = wizard_pos + normalized_direction * max_center_distance;
    }

    Some(pos)
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
#[allow(clippy::too_many_arguments)]
fn apply_guardian_circle_buff(
    commands: &mut Commands,
    circle_pos: Vec3,
    radius: f32,
    temp_hp_amount: f32,
    duration: f32,
    empowerment: f32,
    targets: &mut Query<(Entity, &Transform, &Team), Without<Wizard>>,
    attacker_hit_msg: &mut MessageWriter<GuardianCircleHitAttackerMessage>,
) {
    // Scale values by empowerment
    let scale = empowerment;
    let scaled_temp_hp = temp_hp_amount * scale;
    let scaled_duration = duration * scale;

    for (entity, transform, team) in targets.iter() {
        let distance = transform.translation.distance(circle_pos);

        if distance <= radius {
            // Unit is in range - add or update TemporaryHitPoints
            commands
                .entity(entity)
                .insert(TemporaryHitPoints::new(scaled_temp_hp, scaled_duration));

            // Protective Instincts: Guardian Circle hit an attacker or undead
            if *team == Team::Attackers || *team == Team::Undead {
                attacker_hit_msg.write(GuardianCircleHitAttackerMessage);
            }
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
