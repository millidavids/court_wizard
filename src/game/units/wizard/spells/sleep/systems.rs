use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, GuestWizard, Mana, PrimedSpell, Spell, SpellCaster, LocalWizard, Wizard, WizardInput};
use super::components::SleepIndicator;
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::spell_commands::{GuestCursorPosition, GuestInputState};
use crate::game::units::components::{Corpse, SleepModifier};

/// Result from spell casting logic, used to communicate state back to the wrapper.
struct CastResult {
    /// Whether the spell completed (cast finished and effect spawned).
    completed: bool,
}

/// Local wizard sleep casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_sleep_casting(
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
    mut indicator_query: Query<&mut SleepIndicator>,
    targets_query: Query<(Entity, &Transform), Without<Corpse>>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true, // Run conditions already ensure mouse is held
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::Sleep { return; }

    let cast_result = sleep_casting_logic(
        &input,
        &time,
        wizard_entity,
        wizard_transform,
        wizard,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut indicator_query,
        &targets_query,
        &mut commands,
        &mut meshes,
        &mut materials,
        true, // is_local
    );

    if cast_result.completed {
        mouse_state.left_consumed = true;
    }
}

/// Guest wizard sleep casting -- reads network signals.
#[allow(clippy::too_many_arguments)]
pub fn handle_sleep_casting_guest(
    time: Res<Time>,
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
        With<GuestWizard>,
    >,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SleepIndicator>,
    targets_query: Query<(Entity, &Transform), Without<Corpse>>,
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
    if primed_spell.spell != Spell::Sleep { return; }

    sleep_casting_logic(
        &input,
        &time,
        wizard_entity,
        wizard_transform,
        wizard,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut indicator_query,
        &targets_query,
        &mut commands,
        &mut meshes,
        &mut materials,
        false, // is_local
    );
}

/// Core sleep casting logic -- called by both local and guest systems.
#[allow(clippy::too_many_arguments)]
fn sleep_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    wizard_transform: &Transform,
    wizard: &Wizard,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &mut Query<&mut SleepIndicator>,
    targets_query: &Query<(Entity, &Transform), Without<Corpse>>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    is_local: bool,
) -> CastResult {
    let mut result = CastResult { completed: false };

    // Check for release event
    if input.just_released {
        if let Ok(caster) = caster_query.get(wizard_entity) {
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).despawn();
            }
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return result;
    }

    let Some(mut cursor_world_pos) = input.cursor_pos else {
        return result;
    };

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
    let direction = cursor_world_pos - wizard_pos;
    let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();
    if distance > max_center_distance && distance > 0.001 {
        let normalized_direction = direction / distance;
        cursor_world_pos = wizard_pos + normalized_direction * max_center_distance;
    }

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(constants::MANA_COST)
            {
                if is_local {
                    let circle_entity = spawn_circle_indicator(
                        commands,
                        meshes,
                        materials,
                        cursor_world_pos,
                        primed_spell.empowerment,
                    );
                    commands
                        .entity(wizard_entity)
                        .insert(SpellCaster::with_indicator(circle_entity));
                } else {
                    commands
                        .entity(wizard_entity)
                        .insert(SpellCaster::new());
                }
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());
            if is_local {
                if let Ok(caster) = caster_query.get(wizard_entity)
                    && let Some(indicator_entity) = caster.indicator_entity
                    && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
                {
                    indicator.position = cursor_world_pos;
                }
            }
            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    if is_local {
                        if let Ok(caster) = caster_query.get(wizard_entity)
                            && let Some(indicator_entity) = caster.indicator_entity
                        {
                            if let Ok(indicator) = indicator_query.get(indicator_entity) {
                                let radius = constants::CIRCLE_RADIUS * indicator.empowerment;
                                apply_sleep(
                                    commands,
                                    indicator.position,
                                    radius,
                                    indicator.empowerment,
                                    targets_query,
                                );
                            }
                            commands.entity(indicator_entity).despawn();
                        }
                    } else {
                        // Guest: apply sleep at cursor position directly
                        let radius = constants::CIRCLE_RADIUS * primed_spell.empowerment;
                        apply_sleep(
                            commands,
                            cursor_world_pos,
                            radius,
                            primed_spell.empowerment,
                            targets_query,
                        );
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                    result.completed = true;
                } else {
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        commands.entity(indicator_entity).despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
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
            casting_state.cancel();
        }
    }

    result
}

pub fn update_sleep_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut SleepIndicator, &mut Transform)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        indicator.time_alive += time.delta_secs();
        let pulse = indicator.pulse_scale();
        transform.scale = Vec3::splat(pulse);
        transform.translation.x = indicator.position.x;
        transform.translation.y = constants::CIRCLE_Y_POSITION;
        transform.translation.z = indicator.position.z;
    }
}

fn apply_sleep(
    commands: &mut Commands,
    circle_pos: Vec3,
    radius: f32,
    empowerment: f32,
    targets: &Query<(Entity, &Transform), Without<Corpse>>,
) {
    let duration = constants::SLEEP_DURATION * empowerment;
    let bonus = constants::BONUS_DAMAGE_MULTIPLIER;
    for (entity, transform) in targets.iter() {
        let distance = transform.translation.distance(circle_pos);
        if distance <= radius {
            commands
                .entity(entity)
                .insert(SleepModifier::new(duration, bonus));
        }
    }
}

fn spawn_circle_indicator(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    empowerment: f32,
) -> Entity {
    let radius = constants::CIRCLE_RADIUS * empowerment;
    let circle_mesh = meshes.add(Circle::new(radius));
    let circle_material = materials.add(StandardMaterial {
        base_color: constants::CIRCLE_COLOR,
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
            SleepIndicator::new(position, empowerment),
            OnGameplayScreen,
        ))
        .id()
}

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
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return None;
    };
    if ray.direction.y.abs() < 0.0001 {
        return None;
    }
    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        return None;
    }
    Some(ray.origin + ray.direction * t)
}
