use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, Mana, PrimedSpell, Spell, LocalWizard, WizardInput};
use super::components::ActiveMarkOfDeath;
use super::constants;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::spell_commands::SkipSpellSpawning;
use crate::game::units::components::{Corpse, MarkedForDeathModifier, Team};
use crate::networking::protocol::{NetworkMessage, SpellAction};
use crate::networking::resources::NetworkConnection;

/// Result from spell casting logic, used to communicate state back to the wrapper.
struct CastResult {
    /// Whether the spell completed (cast finished and effect spawned/skipped).
    completed: bool,
    /// Cursor position at time of completion (for network message).
    cursor_pos: Option<Vec3>,
}

/// Local wizard mark of death casting — reads mouse input.
///
/// On the guest (when `SkipSpellSpawning` is present), the casting pipeline
/// runs normally (CastingState, mana, cast bar) but entity spawning is skipped.
/// Instead, a `SpellCast` message is sent to the host.
#[allow(clippy::too_many_arguments)]
pub fn handle_mark_of_death_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<
        (Entity, &Transform, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    enemies_query: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    existing_marks: Query<Entity, With<ActiveMarkOfDeath>>,
    skip_spawning: Option<Res<SkipSpellSpawning>>,
    mut connection: Option<ResMut<NetworkConnection>>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((_wizard_entity, _wizard_transform, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::MarkOfDeath { return; }

    let skip_spawn = skip_spawning.is_some();

    let cast_result = mark_of_death_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut commands,
        &enemies_query,
        &existing_marks,
        skip_spawn,
    );

    if cast_result.completed {
        mouse_state.left_consumed = true;

        if skip_spawn {
            if let (Some(conn), Some(pos)) = (connection.as_mut(), cast_result.cursor_pos) {
                conn.outgoing_messages.push(NetworkMessage::SpellResult(
                    SpellAction::SpellCast {
                        spell: Spell::MarkOfDeath,
                        cursor_pos: [pos.x, pos.y, pos.z],
                        empowerment: primed_spell.empowerment,
                    },
                ));
            }
        }
    }
}

/// Core mark of death casting logic.
///
/// When `skip_spawn` is true, the casting pipeline runs normally but
/// entity spawning is skipped. The cursor position is returned in `CastResult`
/// so the caller can send a network message.
#[allow(clippy::too_many_arguments)]
fn mark_of_death_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    commands: &mut Commands,
    enemies_query: &Query<(Entity, &Transform, &Team), Without<Corpse>>,
    existing_marks: &Query<Entity, With<ActiveMarkOfDeath>>,
    skip_spawn: bool,
) -> CastResult {
    let mut result = CastResult { completed: false, cursor_pos: None };

    // Check for release event
    if input.just_released {
        casting_state.cancel();
        return result;
    }

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(constants::MANA_COST) {
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());
            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    if let Some(cursor_pos) = input.cursor_pos {
                        result.cursor_pos = Some(cursor_pos);

                        if !skip_spawn {
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
                    }
                    result.completed = true;
                }
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
    }

    result
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
