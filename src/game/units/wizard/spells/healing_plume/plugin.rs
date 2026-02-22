use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{HealingPlumeIndicator, HealingPlumeZone};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;

pub struct HealingPlumePlugin;

impl Plugin for HealingPlumePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_healing_plume_casting
                    .run_if(spell_is_primed(Spell::HealingPlume))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                // Guest wizard casting (network signals)
                systems::handle_healing_plume_casting_guest
                    .run_if(guest_spell_is_primed(Spell::HealingPlume))
                    .run_if(guest_input_or_wizard_casting),
                systems::update_healing_plume_indicator
                    .run_if(any_exist::<HealingPlumeIndicator>()),
                (
                    systems::apply_healing_plume_heal,
                    systems::fade_healing_plume_zone,
                    systems::cleanup_healing_plume_zone,
                )
                    .chain()
                    .run_if(any_exist::<HealingPlumeZone>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
