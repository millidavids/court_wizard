//! Core Wall of Fire casting state machine — transitions, mana, and obstacle events.

use super::super::super::super::components::{CastingState, Mana, PrimedSpell, WizardInput};
use super::super::components::{WallOfFireCaster, WallOfFireTalentParams};
use super::super::constants::*;
use super::placement::{WallOfFireCastResult, WallPlacedInfo, wall_obstacle_bounds};
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use bevy::prelude::*;

/// Core Wall of Fire casting logic.
///
/// Handles state machine transitions, mana consumption, and obstacle events.
/// Does NOT manage preview entities — that is the responsibility of the wrapper.
#[allow(clippy::too_many_arguments)]
pub(crate) fn wall_of_fire_casting_logic(
    input: &WizardInput,
    clamped_pos: Option<Vec3>,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster: &mut WallOfFireCaster,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    talent_params: &WallOfFireTalentParams,
    scorched_mult: f32,
) -> WallOfFireCastResult {
    let mut result = WallOfFireCastResult {
        completed: false,
        despawn_preview: false,
        wall_placed: None,
    };

    let Some(clamped_pos) = clamped_pos else {
        return result;
    };

    // Handle release — place fire wall or cancel
    if input.just_released {
        if let Some(anchor) = caster.anchor {
            let diff = Vec3::new(clamped_pos.x - anchor.x, 0.0, clamped_pos.z - anchor.z);
            let length = diff.length();
            let max_len = MAX_WALL_LENGTH * talent_params.max_length_mult;

            if length >= MIN_WALL_LENGTH && mana.can_afford(MANA_COST) {
                let clamped_length = length.min(max_len);
                let forward = diff.normalize();

                mana.consume(MANA_COST);

                let scale = primed_spell.empowerment;
                let fire_duration =
                    FIRE_DURATION * scale * talent_params.duration_mult * scorched_mult;
                let damage = DAMAGE_PER_TICK * scale * talent_params.damage_mult;
                let half_width = WALL_WIDTH / 2.0 * scale * talent_params.width_mult;

                let wall_start = anchor;
                let wall_end = anchor + forward * clamped_length;

                // Notify pathfinding about hazard (for non-twin-walls; twin walls re-notifies)
                if !talent_params.twin_walls {
                    obstacle_events.write(ObstacleChanged {
                        bounds: wall_obstacle_bounds(wall_start, wall_end, half_width),
                        obstacle_type: ObstacleType::Hazard(4.5),
                        shape: Some(ObstacleShape::obb_from_wall(
                            wall_start,
                            wall_end,
                            half_width + OBSTACLE_BUFFER,
                        )),
                        rebuild: true,
                    });
                }

                result.wall_placed = Some(WallPlacedInfo {
                    wall_start,
                    wall_end,
                    half_width,
                    damage,
                    fire_duration,
                    talent_params: talent_params.clone(),
                });
                result.completed = true;
            } else {
                // Too short or can't afford
                result.despawn_preview = true;
            }

            caster.anchor = None;
            casting_state.cancel();
        }
        return result;
    }

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(MANA_COST) {
                caster.anchor = Some(clamped_pos);
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            // Preview update is handled by the local wrapper only
        }
        _ => {}
    }

    result
}
