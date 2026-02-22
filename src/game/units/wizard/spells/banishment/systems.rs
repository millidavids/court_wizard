use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, GuestWizard, Mana, PrimedSpell, Spell, LocalWizard};
use super::constants;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::spell_commands::{GuestCursorPosition, GuestInputState};
use crate::game::units::components::{BanishedModifier, Corpse, Team, WasBanished};

/// Handles banishment casting for both local and guest wizards.
#[allow(clippy::too_many_arguments)]
pub fn handle_banishment_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<
        (Entity, &Transform, &mut CastingState, &mut Mana, &PrimedSpell, Option<&GuestWizard>),
        Or<(With<LocalWizard>, With<GuestWizard>)>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    enemies_query: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<WasBanished>,
            Without<BanishedModifier>,
        ),
    >,
    guest_cursor: Option<Res<GuestCursorPosition>>,
    guest_input: Option<Res<GuestInputState>>,
) {
    // Read local input once before the loop
    let local_released = mouse_left_released.read().next().is_some();

    for (_wizard_entity, _wizard_transform, mut casting_state, mut mana, primed_spell, is_guest) in wizard_query.iter_mut() {
        if primed_spell.spell != Spell::Banishment { continue; }

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
                            && let Some((target_entity, _)) = enemies_query
                                .iter()
                                .filter(|(_, _, team)| {
                                    **team == Team::Attackers || **team == Team::Undead
                                })
                                .filter_map(|(entity, transform, _)| {
                                    let dist = transform.translation.distance(cursor_pos);
                                    if dist <= constants::TARGET_SEARCH_RADIUS {
                                        Some((entity, dist))
                                    } else {
                                        None
                                    }
                                })
                                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                        {
                            let duration = constants::BANISH_DURATION * primed_spell.empowerment;
                            commands
                                .entity(target_entity)
                                .insert((BanishedModifier::new(duration), Visibility::Hidden));
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

/// Ticks banished unit timers and restores them when expired.
pub fn tick_banished_units(
    mut commands: Commands,
    time: Res<Time>,
    mut banished: Query<(Entity, &mut BanishedModifier)>,
) {
    let delta = time.delta_secs();
    for (entity, mut modifier) in &mut banished {
        if modifier.update(delta) {
            // Banishment expired - restore unit
            commands
                .entity(entity)
                .remove::<BanishedModifier>()
                .insert(Visibility::Visible)
                .insert(WasBanished);
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
