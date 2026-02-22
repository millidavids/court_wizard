use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, GuestWizard, Mana, PrimedSpell, Spell, SpellCaster, LocalWizard, Wizard};
use super::components::HasteIndicator;
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::spell_commands::{GuestCursorPosition, GuestInputState};
use crate::game::units::components::HasteModifier;

#[allow(clippy::too_many_arguments)]
pub fn handle_haste_casting(
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
        Or<(With<LocalWizard>, With<GuestWizard>)>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut HasteIndicator>,
    mut targets_query: Query<(Entity, &Transform, Option<&mut HasteModifier>), Without<Wizard>>,
    guest_cursor: Option<Res<GuestCursorPosition>>,
    guest_input: Option<Res<GuestInputState>>,
) {
    let local_released = mouse_left_released.read().next().is_some();

    for (wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell, is_guest) in wizard_query.iter_mut() {
        if primed_spell.spell != Spell::Haste { continue; }

        let is_guest = is_guest.is_some();
        let released = if is_guest {
            guest_input.as_ref().is_some_and(|i| i.just_released)
        } else {
            local_released
        };

        if released {
            if let Ok(caster) = caster_query.get(wizard_entity) {
                if let Some(indicator_entity) = caster.indicator_entity {
                    commands.entity(indicator_entity).despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
            }
            casting_state.cancel();
            continue;
        }

        let cursor_world_pos = if is_guest {
            guest_cursor.as_ref().and_then(|c| c.position)
        } else {
            get_cursor_world_position(&camera_query, &window_query)
        };
        let Some(mut cursor_world_pos) = cursor_world_pos else {
            continue;
        };

        // Clamp cursor to spell range accounting for circle radius
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
                let has_input = if is_guest {
                    guest_input.as_ref().is_some_and(|i| i.just_pressed || i.pressed)
                } else {
                    true
                };
                if has_input && caster_query.get(wizard_entity).is_err() && mana.can_afford(constants::MANA_COST) {
                    let circle_entity = spawn_circle_indicator(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        cursor_world_pos,
                        primed_spell.empowerment,
                    );
                    commands
                        .entity(wizard_entity)
                        .insert(SpellCaster::with_indicator(circle_entity));
                    casting_state.start_cast();
                }
            }
            CastingState::Casting { .. } => {
                casting_state.advance(time.delta_secs());

                if let Ok(caster) = caster_query.get(wizard_entity)
                    && let Some(indicator_entity) = caster.indicator_entity
                    && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
                {
                    indicator.position = cursor_world_pos;
                }

                if casting_state.is_complete(primed_spell.cast_time) {
                    if mana.consume(constants::MANA_COST) {
                        if let Ok(caster) = caster_query.get(wizard_entity)
                            && let Some(indicator_entity) = caster.indicator_entity
                        {
                            if let Ok(indicator) = indicator_query.get(indicator_entity) {
                                let radius = constants::CIRCLE_RADIUS * indicator.empowerment;
                                apply_haste_buff(
                                    &mut commands,
                                    indicator.position,
                                    radius,
                                    indicator.empowerment,
                                    &mut targets_query,
                                );
                            }
                            commands.entity(indicator_entity).despawn();
                        }
                        commands.entity(wizard_entity).remove::<SpellCaster>();
                        casting_state.cancel();
                        if !is_guest {
                            mouse_state.left_consumed = true;
                        }
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
    }
}

pub fn update_haste_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut HasteIndicator, &mut Transform)>,
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

/// Applies haste buff to ALL units in radius (magic is indiscriminate).
fn apply_haste_buff(
    commands: &mut Commands,
    circle_pos: Vec3,
    radius: f32,
    empowerment: f32,
    targets: &mut Query<(Entity, &Transform, Option<&mut HasteModifier>), Without<Wizard>>,
) {
    let modifier = constants::HASTE_MODIFIER * empowerment;
    let duration = constants::HASTE_DURATION * empowerment;

    for (entity, transform, existing_haste) in targets.iter_mut() {
        let distance = transform.translation.distance(circle_pos);
        if distance <= radius {
            if let Some(mut haste) = existing_haste {
                // Refresh duration if already hasted
                haste.refresh(duration);
            } else {
                commands
                    .entity(entity)
                    .insert(HasteModifier::new(modifier, duration));
            }
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
            HasteIndicator::new(position, empowerment),
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
