use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::systems::*;
use crate::state::InGameState;

pub struct FingerOfDeathPlugin;

impl Plugin for FingerOfDeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_finger_of_death_casting
                    .run_if(spell_is_primed(Spell::FingerOfDeath))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                apply_finger_of_death_damage,
                update_finger_of_death_beam_visuals,
                cleanup_finger_of_death_beams,
            )
                .chain()
                .run_if(in_state(InGameState::Running)),
        );
    }
}
