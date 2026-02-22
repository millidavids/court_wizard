use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, GuestWizard, Mana, PrimedSpell, Spell, LocalWizard};
use super::constants;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::spell_commands::{GuestCursorPosition, GuestInputState};
use crate::game::units::components::{AttackTiming, Corpse, Health, PolymorphedModifier};

/// Handles polymorph casting for both local and guest wizards.
#[allow(clippy::too_many_arguments)]
pub fn handle_polymorph_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (Entity, &Transform, &mut CastingState, &mut Mana, &PrimedSpell, Option<&GuestWizard>),
        Or<(With<LocalWizard>, With<GuestWizard>)>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    targets_query: Query<
        (
            Entity,
            &Transform,
            &Health,
            &MeshMaterial3d<StandardMaterial>,
        ),
        (Without<Corpse>, Without<PolymorphedModifier>),
    >,
    guest_cursor: Option<Res<GuestCursorPosition>>,
    guest_input: Option<Res<GuestInputState>>,
) {
    // Read local input once before the loop
    let local_released = mouse_left_released.read().next().is_some();

    for (_wizard_entity, _wizard_transform, mut casting_state, mut mana, primed_spell, is_guest) in wizard_query.iter_mut() {
        if primed_spell.spell != Spell::Polymorph { continue; }

        let is_guest = is_guest.is_some();
        let released = if is_guest {
            guest_input.as_ref().is_some_and(|i| i.just_released)
        } else {
            local_released
        };

        // Check for release event
        if released {
            casting_state.cancel();
            continue;
        }

        match *casting_state {
            CastingState::Resting => {
                let has_input = if is_guest {
                    guest_input.as_ref().is_some_and(|i| i.just_pressed || i.pressed)
                } else {
                    true // Run conditions already ensure mouse is held for local wizard
                };
                if has_input && mana.can_afford(constants::MANA_COST) {
                    casting_state.start_cast();
                }
            }
            CastingState::Casting { .. } => {
                casting_state.advance(time.delta_secs());
                if casting_state.is_complete(primed_spell.cast_time) {
                    let cursor_pos = if is_guest {
                        guest_cursor.as_ref().and_then(|c| c.position)
                    } else {
                        get_cursor_world_position(&camera_query, &window_query)
                    };

                    if mana.consume(constants::MANA_COST) {
                        if let Some(cursor_pos) = cursor_pos
                            && let Some((target_entity, _, target_health, target_material)) =
                                targets_query
                                    .iter()
                                    .filter_map(|(entity, transform, health, material)| {
                                        let dist = transform.translation.distance(cursor_pos);
                                        if dist <= constants::TARGET_SEARCH_RADIUS {
                                            Some((entity, dist, health, material))
                                        } else {
                                            None
                                        }
                                    })
                                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                        {
                            let duration = constants::POLYMORPH_DURATION * primed_spell.empowerment;
                            let original_material = target_material.0.clone();

                            // Create sheep material
                            let sheep_material = materials.add(StandardMaterial {
                                base_color: constants::SHEEP_COLOR,
                                ..default()
                            });

                            commands.entity(target_entity).insert((
                                PolymorphedModifier::new(
                                    duration,
                                    target_health.current,
                                    target_health.max,
                                    original_material,
                                ),
                                MeshMaterial3d(sheep_material),
                                Health::new(constants::SHEEP_HP),
                            ));
                            commands.entity(target_entity).remove::<AttackTiming>();
                        }
                        if !is_guest {
                            mouse_state.left_consumed = true;
                        }
                    }
                    casting_state.cancel();
                }
            }
            CastingState::Channeling { .. } => {
                casting_state.cancel();
            }
        }
    }
}

/// Ticks polymorphed unit timers and restores them when expired.
pub fn tick_polymorphed_units(
    mut commands: Commands,
    time: Res<Time>,
    mut polymorphed: Query<(Entity, &mut PolymorphedModifier, &mut Health)>,
) {
    let delta = time.delta_secs();
    for (entity, mut modifier, mut health) in &mut polymorphed {
        if modifier.update(delta) {
            // Polymorph expired - restore unit
            health.current = modifier.original_health_current;
            health.max = modifier.original_health_max;
            commands.entity(entity).insert((
                MeshMaterial3d(modifier.original_material.clone()),
                AttackTiming::new(),
            ));
            commands.entity(entity).remove::<PolymorphedModifier>();
        }
    }
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
