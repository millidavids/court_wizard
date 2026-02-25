use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, WizardInput,
};
use super::constants;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{BanishedModifier, Corpse, Team, WasBanished};

/// Local wizard banishment casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_banishment_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<
        (
            Entity,
            &Transform,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
        ),
        With<LocalWizard>,
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
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((_wizard_entity, _wizard_transform, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Banishment {
        return;
    }

    let completed = banishment_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut commands,
        &enemies_query,
    );

    if completed {
        mouse_state.left_consumed = true;
    }
}

/// Core banishment casting logic.
#[allow(clippy::too_many_arguments)]
fn banishment_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    commands: &mut Commands,
    enemies_query: &Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<WasBanished>,
            Without<BanishedModifier>,
        ),
    >,
) -> bool {
    // Check for release event
    if input.just_released {
        casting_state.cancel();
        return false;
    }

    let mut completed = false;

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
                    if let Some(cursor_pos) = input.cursor_pos
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
                    completed = true;
                }
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
    }

    completed
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
