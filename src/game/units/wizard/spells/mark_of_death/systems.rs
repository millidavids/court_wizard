use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, Mana, PrimedSpell, Wizard};
use super::components::ActiveMarkOfDeath;
use super::constants;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Corpse, MarkedForDeathModifier, Team};

#[allow(clippy::too_many_arguments)]
pub fn handle_mark_of_death_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<(&mut CastingState, &mut Mana, &PrimedSpell), With<Wizard>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    enemies_query: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    existing_marks: Query<Entity, With<ActiveMarkOfDeath>>,
) {
    let Ok((mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut() else {
        return;
    };

    if mouse_left_released.read().next().is_some() {
        casting_state.cancel();
        return;
    }

    match *casting_state {
        CastingState::Resting => {
            if mana.can_afford(constants::MANA_COST) {
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());
            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    if let Some(cursor_pos) =
                        get_cursor_world_position(&camera_query, &window_query)
                    {
                        // Find nearest enemy to cursor (only Attackers/Undead)
                        if let Some((target_entity, _)) = enemies_query
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
                            // Remove any existing marks
                            for old_mark_entity in existing_marks.iter() {
                                commands
                                    .entity(old_mark_entity)
                                    .remove::<MarkedForDeathModifier>()
                                    .remove::<ActiveMarkOfDeath>();
                            }

                            // Apply new mark
                            let amplification =
                                constants::DAMAGE_AMPLIFICATION * primed_spell.empowerment;
                            let duration = constants::MARK_DURATION * primed_spell.empowerment;
                            commands.entity(target_entity).insert((
                                MarkedForDeathModifier::new(amplification, duration),
                                ActiveMarkOfDeath,
                            ));
                        }
                    }
                    mouse_state.left_consumed = true;
                }
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            casting_state.cancel();
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
