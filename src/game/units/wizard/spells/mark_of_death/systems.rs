use std::cmp::Ordering;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, WizardInput,
};
use super::components::ActiveMarkOfDeath;
use super::constants;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Corpse, MarkedForDeathModifier, Team};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::get_cursor_world_position;
use crate::config::GameConfig;

/// Local wizard mark of death casting — reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_mark_of_death_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<
        (
            Entity,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    enemies_query: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    existing_marks: Query<Entity, With<ActiveMarkOfDeath>>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((_wizard_entity, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::MarkOfDeath {
        return;
    }

    let completed = mark_of_death_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut commands,
        &enemies_query,
        &existing_marks,
    );

    if completed {
        if let Some(pos) = cursor_pos {
            audio::play_sfx(&mut commands, &sfx.mark_of_death_cast, pos, &game_config);
        }
        mouse_state.left_consumed = true;
    }
}

/// Core mark of death casting logic.
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
                    if let Some(cursor_pos) = input.cursor_pos {
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
                            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
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
