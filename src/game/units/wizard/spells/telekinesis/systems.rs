use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, GuestWizard, Mana, PrimedSpell, Spell, SpellCaster, LocalWizard, Wizard};
use super::components::TelekinesisIndicator;
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::drops::components::{FlyingToWizard, IngredientDrop};
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::spell_commands::{GuestCursorPosition, GuestInputState};

/// Handles Telekinesis casting for both local and guest wizards.
///
/// Click near an ingredient drop to start casting. Hold for cast time.
/// On completion, the drop flies to the wizard for collection.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_telekinesis_casting(
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
    drops_query: Query<(Entity, &Transform, &IngredientDrop), Without<FlyingToWizard>>,
    indicator_query: Query<&TelekinesisIndicator>,
    guest_cursor: Option<Res<GuestCursorPosition>>,
    guest_input: Option<Res<GuestInputState>>,
) {
    // Read local input once before the loop
    let local_released = mouse_left_released.read().next().is_some();

    for (wizard_entity, wizard_transform, wizard, mut casting_state, mut mana, primed_spell, is_guest) in wizard_query.iter_mut() {
        if primed_spell.spell != Spell::Telekinesis { continue; }

        let is_guest = is_guest.is_some();
        let released = if is_guest {
            guest_input.as_ref().is_some_and(|i| i.just_released)
        } else {
            local_released
        };

        // Check for release event
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

        // Get cursor world position
        let cursor_world_pos = if is_guest {
            guest_cursor.as_ref().and_then(|c| c.position)
        } else {
            get_cursor_world_position(&camera_query, &window_query)
        };
        let Some(cursor_world_pos) = cursor_world_pos else {
            continue;
        };

        match *casting_state {
            CastingState::Resting => {
                let has_input = if is_guest {
                    guest_input.as_ref().is_some_and(|i| i.just_pressed || i.pressed)
                } else {
                    true // Run conditions already ensure mouse is held for local wizard
                };
                if has_input && caster_query.get(wizard_entity).is_err() && mana.can_afford(constants::MANA_COST) {
                    // Find nearest drop within pickup radius of cursor
                    let Some((drop_entity, drop_transform, _drop)) =
                        find_nearest_drop(&cursor_world_pos, &drops_query)
                    else {
                        continue;
                    };

                    // Check if drop is within wizard's spell range
                    let wizard_pos = wizard_transform.translation;
                    let drop_pos = drop_transform.translation;
                    let wizard_height = wizard_pos.y;
                    let max_ground_radius = if wizard_height < wizard.spell_range {
                        (wizard.spell_range * wizard.spell_range - wizard_height * wizard_height).sqrt()
                    } else {
                        0.0
                    };
                    let dx = drop_pos.x - wizard_pos.x;
                    let dz = drop_pos.z - wizard_pos.z;
                    let ground_distance = (dx * dx + dz * dz).sqrt();
                    if ground_distance > max_ground_radius {
                        continue;
                    }

                    // Spawn indicator on the drop
                    let indicator_entity = spawn_indicator(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        drop_transform.translation,
                        drop_entity,
                    );

                    commands
                        .entity(wizard_entity)
                        .insert(SpellCaster::with_indicator(indicator_entity));

                    casting_state.start_cast();
                }
            }
            CastingState::Casting { .. } => {
                casting_state.advance(time.delta_secs());

                if casting_state.is_complete(primed_spell.cast_time) {
                    // Find the targeted drop via the indicator
                    let target_drop = caster_query
                        .get(wizard_entity)
                        .ok()
                        .and_then(|caster| caster.indicator_entity)
                        .and_then(|indicator_entity| indicator_query.get(indicator_entity).ok())
                        .map(|indicator| indicator.target_drop);

                    if let Some(drop_entity) = target_drop
                        && mana.consume(constants::MANA_COST)
                    {
                        if let Ok((_entity, drop_transform, drop_component)) =
                            drops_query.get(drop_entity)
                        {
                            let start_pos = drop_transform.translation;
                            let total_distance =
                                start_pos.distance(crate::game::constants::WIZARD_POSITION);

                            // Convert drop to flying state
                            commands
                                .entity(drop_entity)
                                .remove::<IngredientDrop>()
                                .insert(FlyingToWizard {
                                    ingredient: drop_component.ingredient,
                                    start_pos,
                                    total_distance,
                                });
                        }

                        // Cleanup indicator and caster
                        if let Ok(caster) = caster_query.get(wizard_entity)
                            && let Some(indicator_entity) = caster.indicator_entity
                        {
                            commands.entity(indicator_entity).despawn();
                        }
                        commands.entity(wizard_entity).remove::<SpellCaster>();
                        casting_state.cancel();
                        if !is_guest {
                            mouse_state.left_consumed = true;
                        }
                    } else {
                        // Out of mana or no valid target
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
                // Telekinesis doesn't channel
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

/// Updates telekinesis indicator visuals during casting.
pub(super) fn update_telekinesis_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut TelekinesisIndicator, &mut Transform)>,
    drops: Query<&Transform, (With<IngredientDrop>, Without<TelekinesisIndicator>)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        indicator.time_alive += time.delta_secs();

        // Follow the drop's position
        if let Ok(drop_transform) = drops.get(indicator.target_drop) {
            transform.translation.x = drop_transform.translation.x;
            transform.translation.y = constants::INDICATOR_Y;
            transform.translation.z = drop_transform.translation.z;
        }

        // Pulse animation
        let pulse = indicator.pulse_scale();
        transform.scale = Vec3::splat(pulse);
    }
}

/// Finds the nearest ingredient drop within PICKUP_RADIUS of the cursor position.
fn find_nearest_drop<'a>(
    cursor_pos: &Vec3,
    drops: &'a Query<(Entity, &Transform, &IngredientDrop), Without<FlyingToWizard>>,
) -> Option<(Entity, &'a Transform, &'a IngredientDrop)> {
    let mut nearest: Option<(Entity, &Transform, &IngredientDrop, f32)> = None;

    for (entity, transform, drop) in drops.iter() {
        let dx = transform.translation.x - cursor_pos.x;
        let dz = transform.translation.z - cursor_pos.z;
        let distance = (dx * dx + dz * dz).sqrt();

        if distance <= constants::PICKUP_RADIUS
            && (nearest.is_none() || distance < nearest.as_ref().expect("checked").3)
        {
            nearest = Some((entity, transform, drop, distance));
        }
    }

    nearest.map(|(e, t, d, _)| (e, t, d))
}

/// Spawns a visual indicator ring around a targeted drop.
fn spawn_indicator(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    target_drop: Entity,
) -> Entity {
    let ring_mesh = meshes.add(Circle::new(constants::INDICATOR_RADIUS));
    let ring_material = materials.add(StandardMaterial {
        base_color: constants::INDICATOR_COLOR,
        unlit: true,
        ..default()
    });

    commands
        .spawn((
            Mesh3d(ring_mesh),
            MeshMaterial3d(ring_material),
            Transform::from_translation(Vec3::new(position.x, constants::INDICATOR_Y, position.z))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            TelekinesisIndicator::new(target_drop),
            OnGameplayScreen,
        ))
        .id()
}

/// Gets cursor world position at Y=0 plane.
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

    let intersection = ray.origin + ray.direction * t;
    Some(intersection)
}
