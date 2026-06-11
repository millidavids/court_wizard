use crate::game::units::wizard::components::{
    CastingState, Mana, PrimedSpell, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::disintegrate::constants;
use bevy::prelude::*;

/// Action the shared logic requests the wrapper to perform on beams.
pub(super) enum BeamAction {
    /// Update existing beam with new origin, direction, length.
    UpdateBeam {
        origin: Vec3,
        direction: Vec3,
        length: f32,
    },
    /// Spawn a new beam.
    SpawnBeam {
        origin: Vec3,
        direction: Vec3,
        length: f32,
        empowerment: f32,
    },
    /// Despawn all beams for this wizard.
    DespawnAll,
    /// No beam action needed.
    None,
}

/// Result from the shared casting logic.
pub(super) struct CastingResult {
    pub(super) beam_action: BeamAction,
}

/// Core disintegrate casting logic.
///
/// Takes extracted data from queries and returns actions for the wrapper to apply.
#[allow(clippy::too_many_arguments)]
pub(super) fn disintegrate_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    wizard: &Wizard,
    has_existing_beam: bool,
    mana_cost_multiplier: f32,
    local_origin: Vec3,
) -> CastingResult {
    let mut result = CastingResult {
        beam_action: BeamAction::None,
    };

    let wizard_pos = local_origin;

    // Check for release
    if input.just_released {
        casting_state.cancel();
        result.beam_action = BeamAction::DespawnAll;
        return result;
    }

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.advance_channel(time.delta_secs());

            // Compose the talent discount with the Arcanorouter dial, but floor the
            // COMBINED discount at 50% so a maxed mana dial + Annihilation/Efficient
            // can't make the held beam near-free (it stacks a burn DoT, so even a weak
            // beam destroys units if held long enough). `consume_raw` skips the dial
            // multiplier since it's already folded into `combined`.
            let combined = (mana_cost_multiplier * mana.cost_multiplier).max(0.5);
            let mana_cost = constants::MANA_COST_PER_SECOND * combined * time.delta_secs();

            if mana.consume_raw(mana_cost) {
                if let Some(target_pos) = input.cursor_pos {
                    let beam_origin =
                        wizard_pos + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                    let to_target = target_pos - beam_origin;
                    let distance = to_target.length();
                    let clamped_target = if distance > wizard.spell_range {
                        beam_origin + to_target.normalize() * wizard.spell_range
                    } else {
                        target_pos
                    };

                    let direction = (clamped_target - beam_origin).normalize();
                    let beam_length = (clamped_target - beam_origin)
                        .length()
                        .min(constants::BEAM_LENGTH);

                    if has_existing_beam {
                        result.beam_action = BeamAction::UpdateBeam {
                            origin: beam_origin,
                            direction,
                            length: beam_length,
                        };
                    } else {
                        result.beam_action = BeamAction::SpawnBeam {
                            origin: beam_origin,
                            direction,
                            length: beam_length,
                            empowerment: primed_spell.empowerment,
                        };
                    }
                }
            } else {
                casting_state.cancel();
                result.beam_action = BeamAction::DespawnAll;
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                casting_state.start_channeling();

                if let Some(target_pos) = input.cursor_pos {
                    let beam_origin =
                        wizard_pos + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                    let to_target = target_pos - beam_origin;
                    let distance = to_target.length();
                    let clamped_target = if distance > wizard.spell_range {
                        beam_origin + to_target.normalize() * wizard.spell_range
                    } else {
                        target_pos
                    };

                    let direction = (clamped_target - beam_origin).normalize();
                    let beam_length = (clamped_target - beam_origin)
                        .length()
                        .min(constants::BEAM_LENGTH);

                    result.beam_action = BeamAction::SpawnBeam {
                        origin: beam_origin,
                        direction,
                        length: beam_length,
                        empowerment: primed_spell.empowerment,
                    };
                }
            }
        }
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && mana.can_afford_raw(
                    constants::MANA_COST_PER_SECOND
                        * (mana_cost_multiplier * mana.cost_multiplier).max(0.5)
                        * 0.1,
                )
            {
                casting_state.start_cast();
            }
        }
    }

    result
}
