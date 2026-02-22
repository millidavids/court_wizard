use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{SpikeGrowthIndicator, SpikeGrowthZone};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;

pub struct SpikeGrowthPlugin;

impl Plugin for SpikeGrowthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_spike_growth_casting
                    .run_if(spell_is_primed(Spell::SpikeGrowth))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                // Guest wizard casting (network signals)
                systems::handle_spike_growth_casting_guest
                    .run_if(guest_spell_is_primed(Spell::SpikeGrowth))
                    .run_if(guest_input_or_wizard_casting),
                systems::update_spike_growth_indicator.run_if(any_exist::<SpikeGrowthIndicator>()),
                (
                    systems::apply_spike_growth_damage,
                    systems::fade_spike_growth_zone,
                    systems::cleanup_spike_growth_zone,
                )
                    .chain()
                    .run_if(any_exist::<SpikeGrowthZone>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
