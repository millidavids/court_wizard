use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::DisintegrateBeam;
use super::systems;
use crate::game::run_conditions::{is_gameplay_running, is_spell_effects_active};

/// Plugin that handles disintegrate spell casting and behavior.
pub struct DisintegratePlugin;

impl Plugin for DisintegratePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input) - runs on both host and guest
                systems::handle_disintegrate_casting
                    .run_if(spell_is_primed(Spell::Disintegrate))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting)
                    .run_if(is_spell_effects_active),
                // Guest wizard casting on host (network signals)
                systems::handle_disintegrate_casting_guest
                    .run_if(guest_spell_is_primed(Spell::Disintegrate))
                    .run_if(guest_input_or_wizard_casting)
                    .run_if(is_gameplay_running),
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
