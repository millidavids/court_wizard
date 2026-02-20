use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::PhantasmalForceIndicator;
use super::systems;
use crate::game::units::components::IllusionDecoy;
use crate::game::run_conditions::is_gameplay_running;

pub struct PhantasmalForcePlugin;

impl Plugin for PhantasmalForcePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_phantasmal_force_casting
                    .run_if(spell_is_primed(Spell::PhantasmalForce))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                systems::update_phantasmal_force_indicator
                    .run_if(any_exist::<PhantasmalForceIndicator>()),
                systems::tick_illusion_decoys.run_if(any_exist::<IllusionDecoy>()),
            )
                .run_if(is_gameplay_running),
        );
    }
}
