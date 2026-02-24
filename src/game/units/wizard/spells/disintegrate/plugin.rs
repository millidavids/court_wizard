use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::DisintegrateBeam;
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;

/// Plugin that handles disintegrate spell casting and behavior.
pub struct DisintegratePlugin;

impl Plugin for DisintegratePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_disintegrate_casting
                    .run_if(spell_is_primed(Spell::Disintegrate))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting)
                    .run_if(is_spell_effects_active),
                (
                    systems::update_beam_visuals,
                    systems::apply_disintegrate_damage,
                    systems::cleanup_beams_on_cancel,
                )
                    .chain()
                    .run_if(any_exist::<DisintegrateBeam>())
                    .run_if(is_spell_effects_active),
            ),
        );
    }
}
