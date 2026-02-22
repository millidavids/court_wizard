use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{ChainLightningArc, ChainLightningBolt, ChainLightningGroup};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;

pub struct ChainLightningPlugin;

impl Plugin for ChainLightningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Local wizard casting (mouse input)
                systems::handle_chain_lightning_casting
                    .run_if(spell_is_primed(Spell::ChainLightning))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                // Guest wizard casting (network signals)
                systems::handle_chain_lightning_casting_guest
                    .run_if(guest_spell_is_primed(Spell::ChainLightning))
                    .run_if(guest_input_or_wizard_casting),
                (
                    systems::process_chain_lightning_bounces,
                    systems::update_chain_lightning_arcs,
                    systems::cleanup_chain_lightning,
                    systems::cleanup_chain_lightning_groups,
                )
                    .chain()
                    .run_if(
                        any_exist::<ChainLightningBolt>()
                            .or(any_exist::<ChainLightningArc>())
                            .or(any_exist::<ChainLightningGroup>()),
                    ),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
